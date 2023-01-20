//! Front end for executing code from a source on a VM.
use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::fs::canonicalize;
use std::io::{BufRead, Read};
use std::path::Path;
use std::sync::{Arc, RwLock};

use flate2::read::GzDecoder;
use once_cell::sync::Lazy;
use tar::Archive as TarArchive;

use crate::compiler::{CompErr, CompErrKind, Compiler};
use crate::modules::std::{BUILTINS, SYSTEM};
use crate::parser::{ParseErr, ParseErrKind, Parser};
use crate::result::{ExeErr, ExeErrKind, ExeResult};
use crate::scanner::{ScanErr, ScanErrKind, Scanner, Token, TokenWithLocation};
use crate::source::{
    source_from_bytes, source_from_file, source_from_stdin, source_from_text, Location,
    Source,
};
use crate::types::gen::obj_ref;
use crate::types::{new, Module, ObjectRef, ObjectTrait};
use crate::vm::{
    CallDepth, Inst, PrintFlags, RuntimeContext, RuntimeErr, RuntimeErrKind,
    VMExeResult, VMState, VM,
};
use crate::{ast, dis};

/// At build time, a compressed archive is created containing the
/// builtin module files (see `build.rs`).
///
/// At runtime, the module file data is read out and stored in a map
/// (lazily). When a builtin module is imported, the file data is read
/// from this map rather than reading from disk.
///
/// The utility of this is that we don't need an install process that
/// copies the builtin module files into some location on the file
/// system based on the location of the current executable or anything
/// like that.
static BUILTIN_MODULES: Lazy<HashMap<String, Vec<u8>>> = Lazy::new(|| {
    let archive_bytes: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/modules.tgz"));
    let decoder = GzDecoder::new(archive_bytes);
    let mut archive = TarArchive::new(decoder);
    let mut modules = HashMap::new();
    for entry in archive.entries().unwrap() {
        let mut entry = entry.unwrap();
        let path: Cow<'_, Path> = entry.path().unwrap();
        let path = path.to_str().unwrap().to_owned();
        let mut result = Vec::new();
        entry.read_to_end(&mut result).unwrap();
        modules.insert(path, result);
    }
    modules
});

pub struct Executor {
    vm: VM,
    argv: Vec<String>,
    incremental: bool,
    dis: bool,
    debug: bool,
    current_file_name: String,
    imports: VecDeque<String>,
}

impl Executor {
    pub fn new(
        max_call_depth: CallDepth,
        argv: Vec<String>,
        incremental: bool,
        dis: bool,
        debug: bool,
    ) -> Self {
        let vm = VM::new(RuntimeContext::new(), max_call_depth);

        Self {
            vm,
            argv,
            incremental,
            dis,
            debug,
            current_file_name: "<none>".to_owned(),
            imports: VecDeque::new(),
        }
    }

    /// Set current file name from `path` if possible.
    fn set_current_file_name(&mut self, path: &Path) {
        self.current_file_name = if let Ok(abs_path) = canonicalize(path) {
            abs_path.to_str().unwrap_or("<unknown>").to_owned()
        } else {
            path.to_str().unwrap_or("<unknown>").to_owned()
        };
    }

    pub fn install_sigint_handler(&mut self) {
        self.vm.install_sigint_handler();
    }

    // Bootstrap -------------------------------------------------------

    /// Bootstrap and return error on failure.
    pub fn bootstrap(&mut self) -> Result<(), ExeErr> {
        {
            let mut system = SYSTEM.write().unwrap();
            let argv = new::tuple(self.argv.iter().map(new::str).collect());
            system.ns_mut().add_obj("argv", argv);
        }

        self.add_module("std.system", SYSTEM.clone())?;
        self.extend_base_module("std.builtins", BUILTINS.clone())?;
        self.extend_base_module("std.system", SYSTEM.clone())?;

        Ok(())
    }

    /// Extend module implemented in Rust with module implemented in
    /// FeInt.
    fn extend_base_module(
        &mut self,
        name: &str,
        base_module_ref: ObjectRef,
    ) -> Result<(), ExeErr> {
        let module = self.load_module(name, true)?;
        let mut base_module = base_module_ref.write().unwrap();
        for (name, val) in module.iter_globals() {
            base_module.ns_mut().add_obj(name, val.clone());
        }
        Ok(())
    }

    // Execute ---------------------------------------------------------

    /// Execute text entered in REPL. REPL execution is different from
    /// the other types of execution where the text or source is
    /// compiled all at once and executed as a script. In the REPL, code
    /// is compiled incrementally as it's entered, which makes it
    /// somewhat more complex to deal with.
    pub fn execute_repl(&mut self, text: &str, module_ref: ObjectRef) -> ExeResult {
        self.current_file_name = "<repl>".to_owned();
        let source = &mut source_from_text(text);
        let ast_module = self.parse_source(source)?;
        let mut compiler = Compiler::new(false);

        // XXX: Nested scopes are necessary to avoid deadlock.
        let (start, comp_result) = {
            let module_read_guard = module_ref.read().unwrap();
            let module_read = module_read_guard.down_to_mod().unwrap();
            (
                module_read.code().len_chunk(),
                compiler.compile_module_to_code(module_read.name(), ast_module),
            )
        };

        let mut code = comp_result.map_err(|err| {
            self.handle_comp_err(&err, source);
            ExeErr::new(ExeErrKind::CompErr(err.kind))
        })?;

        // Assign TOS to _, print it, then pop it to clear the stack
        let last_inst = code.pop_inst();
        if let Some(Inst::Pop) = last_inst {
            let print_flags = PrintFlags::ERR
                | PrintFlags::NL
                | PrintFlags::REPR
                | PrintFlags::NO_NIL;
            code.push_inst(Inst::DeclareVar("_".to_owned()));
            code.push_inst(Inst::AssignVar("_".to_owned()));
            code.push_inst(Inst::Print(print_flags));
        } else {
            let last_inst = match last_inst {
                Some(inst) => format!("{inst:?}"),
                None => "[EMPTY CHUNK]".to_owned(),
            };
            panic!("Expected module chunk to end with POP; got {}", last_inst);
        }

        // XXX: Rather than extending the module's code object, perhaps
        //      it would be better to compile INTO the existing module.
        //      This would required passing the module or its code obj
        //      into the compiler.
        {
            let mut module_write = module_ref.write().unwrap();
            let module_write = module_write.down_to_mod_mut().unwrap();
            module_write.code_mut().extend(code);
        }

        let module_read_guard = module_ref.read().unwrap();
        let module_read = module_read_guard.down_to_mod().unwrap();
        self.execute_module(module_read, start, self.debug, source)
    }

    /// Execute source from file as script.
    pub fn execute_file(&mut self, file_path: &Path) -> ExeResult {
        match source_from_file(file_path) {
            Ok(mut source) => {
                self.set_current_file_name(file_path);
                self.execute_script_from_source(&mut source)
            }
            Err(err) => {
                let message = format!("{}: {err}", file_path.display());
                Err(ExeErr::new(ExeErrKind::CouldNotReadSourceFile(message)))
            }
        }
    }

    /// Execute stdin as script.
    pub fn execute_stdin(&mut self) -> ExeResult {
        self.current_file_name = "<stdin>".to_owned();
        let mut source = source_from_stdin();
        self.execute_script_from_source(&mut source)
    }

    /// Execute text as script.
    pub fn execute_text(&mut self, text: &str) -> ExeResult {
        self.current_file_name = "<text>".to_owned();
        let mut source = source_from_text(text);
        self.execute_script_from_source(&mut source)
    }

    /// Execute source as script. The source will be compiled into a
    /// module. If the module contains a global `$main` function, it
    /// will be run automatically.
    fn execute_script_from_source<T: BufRead>(
        &mut self,
        source: &mut Source<T>,
    ) -> ExeResult {
        let module = self.compile_script("$main", source)?;
        let module_ref = obj_ref!(module);

        self.add_module("$main", module_ref.clone())?;

        let module = module_ref.read().unwrap();
        let module = module.down_to_mod().unwrap();

        if self.dis {
            let mut disassembler = dis::Disassembler::new();
            disassembler.disassemble(module.code());
            if self.debug {
                println!();
                self.display_stack();
            }
            Ok(VMState::Halted(0))
        } else {
            self.execute_module(module, 0, self.debug, source)
        }
    }

    /// Execute a module.
    pub fn execute_module<T: BufRead>(
        &mut self,
        module: &Module,
        start: usize,
        debug: bool,
        source: &mut Source<T>,
    ) -> ExeResult {
        self.load_imported_modules()?;
        let result = self.vm.execute_module(module, start);
        if debug {
            self.display_stack();
            self.display_vm_state(&result);
        }
        match result {
            Ok(()) => Ok(self.vm.state.clone()),
            Err(err) => {
                if let RuntimeErrKind::Exit(_) = err.kind {
                    Err(ExeErr::new(ExeErrKind::RuntimeErr(err.kind)))
                } else {
                    let start = self.vm.loc().0;
                    let line = source
                        .get_line(start.line)
                        .unwrap_or("<source line not available>");
                    self.print_err_line(start.line, line);
                    self.handle_runtime_err(&err);
                    Err(ExeErr::new(ExeErrKind::RuntimeErr(err.kind)))
                }
            }
        }
    }

    // Parsing ---------------------------------------------------------

    /// Parse source text, file, etc into AST module node.
    fn parse_source<T: BufRead>(
        &mut self,
        source: &mut Source<T>,
    ) -> Result<ast::Module, ExeErr> {
        let scanner = Scanner::new(source);
        let mut parser = Parser::new(scanner);
        match parser.parse() {
            Ok(ast_module) => {
                self.find_imports(&ast_module);
                Ok(ast_module)
            }
            Err(err) => {
                if let ParseErrKind::ScanErr(scan_err) = err.kind {
                    self.handle_scan_err(&scan_err, source);
                    Err(ExeErr::new(ExeErrKind::ScanErr(scan_err.kind)))
                } else {
                    self.handle_parse_err(&err, source);
                    Err(ExeErr::new(ExeErrKind::ParseErr(err.kind)))
                }
            }
        }
    }

    // Compilation -----------------------------------------------------

    /// Compile AST module node into script module object.
    fn compile_script<T: BufRead>(
        &mut self,
        name: &str,
        source: &mut Source<T>,
    ) -> Result<Module, ExeErr> {
        let ast_module = self.parse_source(source)?;
        let mut compiler = Compiler::new(true);
        let module = compiler
            .compile_script(
                name,
                self.current_file_name.as_str(),
                ast_module,
                &self.argv,
            )
            .map_err(|err| {
                self.handle_comp_err(&err, source);
                ExeErr::new(ExeErrKind::CompErr(err.kind))
            })?;
        Ok(module)
    }

    /// Compile AST module node into module object.
    fn compile_module<T: BufRead>(
        &mut self,
        name: &str,
        source: &mut Source<T>,
        check_names: bool,
    ) -> Result<Module, ExeErr> {
        let ast_module = self.parse_source(source)?;
        let mut compiler = Compiler::new(check_names);
        let module = compiler
            .compile_module(name, self.current_file_name.as_str(), ast_module)
            .map_err(|err| {
                self.handle_comp_err(&err, source);
                ExeErr::new(ExeErrKind::CompErr(err.kind))
            })?;
        Ok(module)
    }

    // Modules/Imports -------------------------------------------------

    /// Add a module to `system.modules`.
    pub fn add_module(&mut self, name: &str, module: ObjectRef) -> Result<(), ExeErr> {
        let system = SYSTEM.read().unwrap();
        let modules = system.get_attr("modules", SYSTEM.clone());
        let modules = modules.write().unwrap();
        if let Some(modules) = modules.down_to_map() {
            modules.add(name, module);
            Ok(())
        } else {
            let msg = format!("Expected system.modules to be a Map; got {modules}");
            Err(ExeErr::new(ExeErrKind::Bootstrap(msg)))
        }
    }

    /// Find imports at the top level of the specified AST module.
    fn find_imports(&mut self, ast_module: &ast::Module) {
        let mut visitor = ImportVisitor::new();
        visitor.visit_module(ast_module);
        for name in visitor.imports {
            if !self.imports.contains(&name) {
                self.imports.push_back(name);
            }
        }
    }

    /// Load FeInt module from file system.
    fn load_module(&mut self, name: &str, check_names: bool) -> Result<Module, ExeErr> {
        let mut segments = name.split('.');

        if let Some(first) = segments.next() {
            return if first == "std" {
                self.load_builtin_module(name, check_names)
            } else {
                Err(ExeErr::new(ExeErrKind::ModuleNotFound(
                    format!("{name}: Only std modules are supported currently"),
                    None,
                )))
            };
        } else {
            unreachable!("Empty module name should not be possible");
        };

        // TODO: Load site or user module
    }

    fn load_builtin_module(
        &mut self,
        name: &str,
        check_names: bool,
    ) -> Result<Module, ExeErr> {
        if let Some(bytes) = BUILTIN_MODULES.get(name) {
            self.set_current_file_name(Path::new(name));
            let mut source = source_from_bytes(bytes);
            let mut module = self.compile_module(name, &mut source, check_names)?;
            self.execute_module(&module, 0, self.debug, &mut source)?;
            for (name, obj) in self.vm.ctx.globals().iter() {
                module.add_global(name, obj.clone());
            }
            Ok(module)
        } else {
            Err(ExeErr::new(ExeErrKind::ModuleNotFound(name.to_owned(), None)))
        }
    }

    fn load_imported_modules(&mut self) -> Result<(), ExeErr> {
        let system = SYSTEM.read().unwrap();
        let modules_ref = system.get_attr("modules", SYSTEM.clone());
        while let Some(name) = self.imports.pop_front() {
            let modules_guard = modules_ref.read().unwrap();
            let modules = modules_guard.down_to_map().unwrap();
            if !modules.contains_key(&name) {
                drop(modules_guard); // XXX: Prevent deadlock in recursive calls
                let module = self.load_module(&name, true)?;
                let modules_write = modules_ref.write().unwrap();
                let modules_write = modules_write.down_to_map().unwrap();
                modules_write.add(&name, obj_ref!(module));
            }
        }
        Ok(())
    }

    // Error Handling --------------------------------------------------

    fn print_err_line(&self, line_no: usize, line: &str) {
        let file_name = self.current_file_name.as_str();
        let line = line.trim_end();
        eprintln!("\n  Error in {file_name} on line {line_no}:\n\n    |\n    |{line}");
    }

    fn print_err_message(&self, message: String, start: Location, end: Location) {
        if !message.is_empty() {
            let start_pos = if start.col == 0 { 0 } else { start.col - 1 };
            let marker = if start == end {
                format!("{:>start_pos$}^", "")
            } else {
                let end_pos = if end.col == 0 { 0 } else { end.col - start.col };
                format!("{:>start_pos$}^{:^>end_pos$}", "", "")
            };
            eprintln!("    |{marker}\n\n  {message}\n");
        }
    }

    fn handle_scan_err<T: BufRead>(&self, err: &ScanErr, source: &Source<T>) {
        use ScanErrKind::*;
        let ignore = self.incremental
            && matches!(
                &err.kind,
                ExpectedBlock
                    | ExpectedIndentedBlock(_)
                    | UnmatchedOpeningBracket(_)
                    | UnterminatedStr(_)
            );
        if ignore {
            return;
        }
        self.print_err_line(
            source.line_no,
            source.get_current_line().unwrap_or("<none>"),
        );
        let mut loc = err.location;
        let col = loc.col;
        let mut message = match &err.kind {
            UnexpectedChar(c) => {
                format!("Syntax error: Unexpected character at column {}: '{}'", col, c)
            }
            UnmatchedOpeningBracket(_) => {
                format!("Unmatched open bracket at {loc}")
            }
            UnterminatedStr(_) => {
                format!("Syntax error: Unterminated string literal at {loc}")
            }
            InvalidIndent(num_spaces) => {
                format!("Syntax error: Invalid indent with {num_spaces} spaces (should be a multiple of 4)")
            }
            ExpectedBlock => "Syntax error: Expected block".to_string(),
            ExpectedIndentedBlock(_) => {
                "Syntax error: Expected indented block".to_string()
            }
            UnexpectedIndent(_) => "Syntax error: Unexpected indent".to_string(),
            WhitespaceAfterIndent | UnexpectedWhitespace => {
                "Syntax error: Unexpected whitespace".to_string()
            }
            InvalidLabel(msg) => {
                format!("Syntax error: Invalid label: {msg}")
            }
            FormatStrErr(err) => {
                use crate::format::FormatStrErr::*;
                match err {
                    EmptyExpr(pos) => {
                        loc = Location::new(loc.line, loc.col + pos + 2);
                        "Syntax error in format string: expected expression".to_string()
                    }
                    UnmatchedOpeningBracket(pos) => {
                        loc = Location::new(loc.line, loc.col + pos + 2);
                        "Unmatched opening bracket in format string".to_string()
                    }
                    UnmatchedClosingBracket(pos) => {
                        loc = Location::new(loc.line, loc.col + pos + 2);
                        "Unmatched closing bracket in format string".to_string()
                    }
                    ScanErr(pos, _) => {
                        loc = Location::new(loc.line, loc.col + pos + 2);
                        "Error while scanning format string".to_string()
                    }
                }
            }
            kind => {
                format!("Unhandled scan error at {loc}: {kind:?}")
            }
        };
        if self.debug {
            message = format!("SCAN ERROR: {message}");
        }
        self.print_err_message(message, loc, loc);
    }

    fn handle_parse_err<T: BufRead>(&self, err: &ParseErr, source: &Source<T>) {
        use ParseErrKind::*;
        if self.incremental && matches!(&err.kind, ExpectedBlock(_)) {
            return;
        }
        let loc = err.loc();
        self.print_err_line(loc.line, source.get_line(loc.line).unwrap_or("<none>"));
        let mut message = match &err.kind {
            ScanErr(_) => {
                unreachable!("Handle ScanErr before calling handle_parse_err")
            }
            UnexpectedToken(TokenWithLocation {
                token: Token::EndOfStatement, ..
            }) => {
                format!("Syntax error at {loc} (unexpected end of statement)")
            }
            UnexpectedToken(token) => {
                format!("Parse error: unexpected token at {loc}: {:?}", token.token)
            }
            ExpectedBlock(loc) => {
                format!("Parse error: expected indented block at {loc}")
            }
            ExpectedToken(loc, token) => {
                format!("Parse error: expected token '{token}' at {loc}")
            }
            ExpectedExpr(loc) => {
                format!("Parse error: expected expression at {loc}")
            }
            ExpectedIdent(loc) => {
                format!("Parse error: expected identifier at {loc}")
            }
            UnexpectedBreak(loc) => {
                format!(
                    "Parse error: unexpected break at {loc} (break must be in a loop)"
                )
            }
            UnexpectedContinue(loc) => {
                format!("Parse error: unexpected continue at {loc} (continue must be in a loop)")
            }
            UnexpectedReturn(loc) => {
                format!("Parse error: unexpected return at {loc} (return must be in a function)")
            }
            InlineMatchNotAllowed(_) => {
                "Parse error: match blocks must be indented".to_string()
            }
            MatchDefaultMustBeLast(_) => {
                "Parse error: extra match arm found after default match arm".to_string()
            }
            SyntaxErr(loc) => format!("Syntax error at {loc}"),
            kind => format!("Unhandled parse error: {:?}", kind),
        };
        if self.debug {
            message = format!("PARSE ERROR: {message}");
        }
        self.print_err_message(message, loc, loc);
    }

    fn handle_comp_err<T: BufRead>(&self, err: &CompErr, source: &Source<T>) {
        use CompErrKind::*;
        if self.incremental && matches!(&err.kind, LabelNotFoundInScope(..)) {
            return;
        }
        let (start, end) = err.loc();
        self.print_err_line(
            start.line,
            source.get_line(start.line).unwrap_or("<none>"),
        );
        let message = match &err.kind {
            NameNotFound(name, ..) =>format!("Name not found: {name}"),
            LabelNotFoundInScope(name, ..) => format!("label not found in scope: {name}"),
            CannotJumpOutOfFunc(name, ..) => format!(
                "cannot jump out of function: label {name} not found or defined in outer scope"
            ),
            DuplicateLabelInScope(name, ..) => format!("duplicate label in scope: {name}"),
            ExpectedIdent(..) => {
                "expected identifier".to_string()
            },
            CannotAssignSpecialIdent(name, ..) => {
                format!("cannot assign to special name: {name}")
            }
            MainMustBeFunc(..) => {
                "$main must be a function".to_owned()
            }
            GlobalNotFound(name, ..) => {
                format!("global var not found: {name}")
            }
            VarArgsMustBeLast(..) => {
                "var args must be last in parameter list".to_owned()
            }
            Print(msg, ..) => {
                format!("$print error: {msg}")
            }
        };
        let message = format!("COMPILATION ERROR: {message}");
        self.print_err_message(message, start, end);
    }

    fn handle_runtime_err(&self, err: &RuntimeErr) {
        use RuntimeErrKind::*;
        let (start, end) = self.vm.loc();
        let mut message = match &err.kind {
            AssertionFailed(message) => {
                if message.is_empty() {
                    "Assertion failed".to_string()
                } else {
                    format!("Assertion failed: {message}")
                }
            }
            RecursionDepthExceeded(max_call_depth) => {
                format!(
                    "Maximum recursion depth of {max_call_depth} was exceeded; use the \
                    --max-call-depth option to raise the limit"
                )
            }
            NameErr(message) => format!("Name error: {message}"),
            TypeErr(message) => format!("Type error: {message}"),
            NotCallable(type_name) => format!("Object is not callable: {type_name}"),
            kind => format!("Unhandled runtime error: {}", kind),
        };
        if self.debug {
            message = format!("RUNTIME ERROR: {message}");
        }
        self.print_err_message(message, start, end);
    }

    // Miscellaneous ---------------------------------------------------

    pub(crate) fn display_stack(&self) {
        eprintln!("{:=<79}", "STACK ");
        self.vm.display_stack();
    }

    fn display_vm_state(&self, result: &VMExeResult) {
        eprintln!("\n{:=<79}", "VM STATE ");
        eprintln!("{:?}", result);
    }
}

/// Find import statements in module AST.
///
/// XXX: Currently, this only looks at top level statements... but
///      perhaps import should only be allowed at the top level anyway?
struct ImportVisitor {
    imports: Vec<String>,
}

impl ImportVisitor {
    fn new() -> Self {
        Self { imports: vec![] }
    }

    fn visit_module(&mut self, node: &ast::Module) {
        self.visit_statements(&node.statements)
    }

    fn visit_statements(&mut self, statements: &[ast::Statement]) {
        statements.iter().for_each(|s| self.visit_statement(s));
    }

    fn visit_statement(&mut self, statement: &ast::Statement) {
        if let ast::StatementKind::Import(name) = &statement.kind {
            if !self.imports.contains(name) {
                self.imports.push(name.to_owned());
            }
        }
    }
}
