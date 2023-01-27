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
use crate::modules::std::{self as stdlib, BUILTINS};
use crate::modules::{add_module, get_module, maybe_get_module};
use crate::parser::{ParseErr, ParseErrKind, Parser};
use crate::result::ExeErrKind::ModuleNotFound;
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
/// builtin .fi module files (see `build.rs`).
///
/// At runtime, the module file data is read out and stored in a map
/// (lazily). When a builtin module is imported, the file data is read
/// from this map rather than reading from disk.
///
/// The utility of this is that we don't need an install process that
/// copies the builtin module files into some location on the file
/// system based on the location of the current executable or anything
/// like that.
static BUILTIN_FI_MODULES: Lazy<HashMap<String, Vec<u8>>> = Lazy::new(|| {
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
        let vm = VM::new(RuntimeContext::default(), max_call_depth);

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
        // Add the `builtins` module first because any other module may
        // rely on it, including `system`.
        self.extend_builtin_module(BUILTINS.clone(), "std.builtins")?;
        add_module("std.builtins", BUILTINS.clone());

        // Add the `system` module next because other modules may rely
        // on it (except for `builtins`), and its where we store system
        // information, such as loaded modules, `argv`, etc.
        let system_ref = self.load_module("std.system")?;
        self.add_module("std.system", system_ref.clone());

        // This may look redundant, but it adds the builtins module to
        // `system.modules`, which we can't do above because the
        // `system` module hasn't been loaded/created/registered yet.
        self.add_module("std.builtins", BUILTINS.clone());

        // Set `system.argv` before adding any other modules in case
        // it's used early (i.e., during import).
        {
            let mut system = system_ref.write().unwrap();
            system.ns_mut().insert("argv", new::argv_tuple(&self.argv));
        }

        self.add_module("std.proc", stdlib::PROC.clone());

        Ok(())
    }

    /// Extend builtin module (implemented in Rust) with global objects
    /// from corresponding FeInt module.
    fn extend_builtin_module(
        &mut self,
        base_module: ObjectRef,
        name: &str,
    ) -> Result<(), ExeErr> {
        let fi_module = self.load_module(name)?;
        let fi_module = fi_module.read().unwrap();
        let fi_module = fi_module.down_to_mod().unwrap();
        let mut base_module = base_module.write().unwrap();
        for (name, val) in fi_module.iter_globals() {
            base_module.ns_mut().insert(name, val.clone());
        }
        Ok(())
    }

    // Execute ---------------------------------------------------------

    /// Execute text entered in REPL. REPL execution is different from
    /// the other types of execution where the text or source is
    /// compiled all at once and executed as a script. In the REPL, code
    /// is compiled incrementally as it's entered, which makes it
    /// somewhat more complex to deal with.
    pub fn execute_repl(&mut self, text: &str, module: ObjectRef) -> ExeResult {
        self.current_file_name = "<repl>".to_owned();

        // XXX: Nested scopes are necessary to avoid deadlocks.
        let (start, global_names) = {
            let module = module.read().unwrap();
            let module = module.down_to_mod().unwrap();
            (
                module.code().len_chunk(),
                module.iter_globals().map(|(n, _)| n.clone()).collect(),
            )
        };

        let source = &mut source_from_text(text);
        let ast_module = self.parse_source(source)?;
        let mut compiler = Compiler::new(global_names);
        let comp_result = compiler.compile_module_to_code("$repl", ast_module);

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

        {
            let mut module = module.write().unwrap();
            let module = module.down_to_mod_mut().unwrap();
            module.code_mut().extend(code);
        }

        let vm_state = {
            let module = module.read().unwrap();
            let module = module.down_to_mod().unwrap();
            self.execute_module(module, start, source, false)?
        };

        {
            let mut module = module.write().unwrap();
            let module = module.down_to_mod_mut().unwrap();
            for (name, obj) in self.vm.ctx.globals().iter() {
                module.add_global(name, obj.clone());
            }
        }

        Ok(vm_state)
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
        let module = self.compile_module("$main", source)?;
        let module_ref = obj_ref!(module);
        self.add_module("$main", module_ref.clone());
        let module = module_ref.read().unwrap();
        let module = module.down_to_mod().unwrap();
        self.execute_module(module, 0, source, true)
    }

    pub fn execute_module_as_script(&mut self, name: &str) -> ExeResult {
        let module = self.get_or_add_module(name)?;
        let module = module.read().unwrap();
        let module = module.down_to_mod().unwrap();
        self.execute_module(module, 0, &mut source_from_bytes(&vec![]), true)
    }

    /// Execute a module.
    ///
    /// NOTE: *All* execution should go through here for standardized
    ///       handling of debugging, disassembly, and errors.
    pub fn execute_module<T: BufRead>(
        &mut self,
        module: &Module,
        start: usize,
        source: &mut Source<T>,
        is_main: bool,
    ) -> ExeResult {
        if self.dis && is_main {
            let mut disassembler = dis::Disassembler::new();
            disassembler.disassemble(module.code());
            if self.debug {
                self.display_stack();
            }
            return Ok(VMState::Halted(0));
        }

        self.load_imported_modules()?;

        let mut result = self.vm.execute_module(module, start);

        if result.is_ok() && is_main {
            if let Some(main) = module.get_main() {
                let main = main.read().unwrap();
                let args = self.argv.iter().map(new::str).collect();
                if let Some(main) = main.down_to_func() {
                    result = self
                        .vm
                        .call_func(main, None, args, None)
                        .and_then(|_| self.vm.halt_top());
                } else if let Some(main) = main.down_to_builtin_func() {
                    result = self
                        .vm
                        .call_builtin_func(main, None, args)
                        .and_then(|_| self.vm.halt_top());
                }
            }
        }

        if self.debug {
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

    /// Compile AST module node into module object.
    fn compile_module<T: BufRead>(
        &mut self,
        name: &str,
        source: &mut Source<T>,
    ) -> Result<Module, ExeErr> {
        let ast_module = self.parse_source(source)?;
        let mut compiler = Compiler::default();
        let module = compiler
            .compile_module(name, self.current_file_name.as_str(), ast_module)
            .map_err(|err| {
                self.handle_comp_err(&err, source);
                ExeErr::new(ExeErrKind::CompErr(err.kind))
            })?;
        Ok(module)
    }

    // Modules/Imports -------------------------------------------------

    /// Load .fi module from file system and compile it to a `Module`.
    ///
    /// XXX: This will load the module regardless of whether it has
    ///      already been loaded.
    fn load_module(&mut self, name: &str) -> Result<ObjectRef, ExeErr> {
        // TODO: Handle non-std modules
        if let Some(file_data) = BUILTIN_FI_MODULES.get(name) {
            self.set_current_file_name(Path::new(&format!("<{name}>")));
            let mut source = source_from_bytes(file_data);
            let mut module = self.compile_module(name, &mut source)?;
            self.execute_module(&module, 0, &mut source, false)?;
            for (name, obj) in self.vm.ctx.globals().iter() {
                module.add_global(name, obj.clone());
            }
            Ok(obj_ref!(module))
        } else {
            Err(ExeErr::new(ModuleNotFound(name.to_owned())))
        }
    }

    /// Add a module to both `MODULES` and `system.modules`.
    pub fn add_module(&mut self, name: &str, module: ObjectRef) {
        add_module(name, module.clone());
        let system_ref = get_module("std.system");
        let system = system_ref.read().unwrap();
        let modules = system.get_attr("modules", system_ref.clone());
        let modules = modules.write().unwrap();
        let modules = modules.down_to_map().unwrap();
        modules.insert(name, module);
    }

    /// Get module from `MODULES` (the `system.modules` mirror).
    fn get_module(&mut self, name: &str) -> Result<ObjectRef, ExeErr> {
        if let Some(module) = maybe_get_module(name) {
            Ok(module)
        } else {
            Err(ExeErr::new(ModuleNotFound(name.to_owned())))
        }
    }

    /// Get module or load it from file system and add it to both
    /// `MODULES` and `system.modules`.
    fn get_or_add_module(&mut self, name: &str) -> Result<ObjectRef, ExeErr> {
        if let Ok(module) = self.get_module(name) {
            Ok(module)
        } else {
            let module = self.load_module(name)?;
            self.add_module(name, module.clone());
            Ok(module)
        }
    }

    /// Find imports at the top level of the specified AST module.
    fn find_imports(&mut self, ast_module: &ast::Module) {
        let mut visitor = ast::visitors::ImportVisitor::new();
        visitor.visit_module(ast_module);
        for name in visitor.imports() {
            if !self.imports.contains(name) {
                self.imports.push_back(name.clone());
            }
        }
    }

    /// Load modules imported by the current module.
    fn load_imported_modules(&mut self) -> Result<(), ExeErr> {
        while let Some(name) = self.imports.pop_front() {
            self.get_or_add_module(&name)?;
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
            CannotReassignSpecialIdent(name, ..) => {
                format!("cannot reassign special name: {name}")
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
