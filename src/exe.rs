//! Front end for executing code from a source on a VM.
use std::io::BufRead;
use std::path::Path;

use crate::compiler::{CompErr, CompErrKind, Compiler};
use crate::config::CONFIG;
use crate::modules;
use crate::parser::{ParseErr, ParseErrKind, Parser};
use crate::result::{ExeErr, ExeErrKind, ExeResult};
use crate::scanner::{ScanErr, ScanErrKind, Scanner, Token, TokenWithLocation};
use crate::types::Module;
use crate::util::{
    source_from_file, source_from_stdin, source_from_text, Location, Source,
};
use crate::vm::{
    CallDepth, Code, Inst, RuntimeContext, RuntimeErr, RuntimeErrKind, VMExeResult,
    VMState, DEFAULT_MAX_CALL_DEPTH, VM,
};
use crate::{ast, dis};

pub struct Executor {
    vm: VM,
    argv: Vec<String>,
    incremental: bool,
    dis: bool,
    debug: bool,
    current_file_name: String,
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
        let current_file_name = "<none>".to_owned();
        modules::init_system_module(&argv);
        Self { vm, argv, incremental, dis, debug, current_file_name }
    }

    pub fn for_add_module() -> Self {
        let config = CONFIG.read().unwrap();
        let max_call_depth = match config.get_usize("max_call_depth") {
            Ok(depth) => depth,
            Err(err) => {
                log::warn!("{err}");
                DEFAULT_MAX_CALL_DEPTH
            }
        };
        drop(config);
        let vm = VM::new(RuntimeContext::new(), max_call_depth);
        let current_file_name = "<none>".to_owned();
        Self {
            vm,
            argv: vec![],
            incremental: false,
            dis: false,
            debug: false,
            current_file_name,
        }
    }

    pub fn halt(&mut self) {
        self.vm.halt();
    }

    pub fn install_sigint_handler(&mut self) {
        self.vm.install_sigint_handler();
    }

    pub fn assign_top(&mut self, name: &str) {
        let code = Code::with_chunk(vec![
            Inst::DeclareVar(name.to_owned()),
            Inst::AssignVar(name.to_owned()),
        ]);
        if let Err(err) = self.vm.execute_code(&code, 0) {
            eprintln!("Could not assign TOS to {name}: {err}");
        }
    }

    /// Execute text entered in REPL. REPL execution is different from
    /// the other types of execution where the text or source is
    /// compiled all at once and executed as a script. In the REPL, code
    /// is compiled incrementally as it's entered, which makes it
    /// somewhat more complex to deal with.
    pub fn execute_repl(&mut self, text: &str, module: &mut Module) -> ExeResult {
        self.current_file_name = "<repl>".to_owned();
        let source = &mut source_from_text(text);
        let ast_module = self.parse_source(source)?;
        let mut compiler = Compiler::new(false);

        let comp_result = compiler.compile_module_to_code(module.name(), ast_module);
        let code = comp_result.map_err(|err| {
            self.handle_comp_err(&err, source);
            ExeErr::new(ExeErrKind::CompErr(err.kind))
        })?;

        // XXX: Rather than extending the module's code object, perhaps
        //      it would be better to compile INTO the existing module.
        //      This would required passing the module or its code obj
        //      into the compiler.
        let start = module.code().len_chunk();
        module.code_mut().extend(code);

        self.execute_module(module, start, self.debug, source)
    }

    /// Execute source from file as script.
    pub fn execute_file(&mut self, file_path: &Path) -> ExeResult {
        match source_from_file(file_path) {
            Ok(mut source) => {
                self.current_file_name =
                    file_path.to_str().unwrap_or("<unknown>").to_owned();
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
        let mut main_module = self.compile_script("$main", source)?;
        if self.dis {
            let mut disassembler = dis::Disassembler::new();
            disassembler.disassemble(main_module.code());
            if self.debug {
                println!();
                self.display_stack();
            }
            Ok(VMState::Halted(0))
        } else {
            self.execute_module(&mut main_module, 0, self.debug, source)
        }
    }

    /// Execute a module.
    pub fn execute_module<T: BufRead>(
        &mut self,
        module: &mut Module,
        start: usize,
        debug: bool,
        source: &mut Source<T>,
    ) -> ExeResult {
        let result = self.vm.execute_module(module, start);
        if debug {
            self.display_stack();
            self.display_vm_state(&result);
        }
        match result {
            Ok(state) => {
                for (name, obj) in self.vm.ctx.globals().iter() {
                    module.add_global(name, obj.clone());
                }
                Ok(state)
            }
            Err(err) => {
                if let RuntimeErrKind::Exit(code) = err.kind {
                    Ok(VMState::Halted(code))
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

    /// Compile AST module node into module object.
    fn compile_script<T: BufRead>(
        &self,
        name: &str,
        source: &mut Source<T>,
    ) -> Result<Module, ExeErr> {
        let ast_module = self.parse_source(source)?;
        let mut compiler = Compiler::new(true);
        let module =
            compiler.compile_script(name, ast_module, &self.argv).map_err(|err| {
                self.handle_comp_err(&err, source);
                ExeErr::new(ExeErrKind::CompErr(err.kind))
            })?;
        Ok(module)
    }

    pub fn load_module(
        &mut self,
        name: &str,
        file_path: &Path,
    ) -> Result<Module, ExeErr> {
        match source_from_file(file_path) {
            Ok(mut source) => {
                self.current_file_name =
                    file_path.to_str().unwrap_or("<unknown>").to_owned();
                match self.compile_module(name, &mut source) {
                    Ok(mut module) => {
                        match self.execute_module(&mut module, 0, false, &mut source) {
                            Ok(_) => Ok(module),
                            Err(err) => Err(err),
                        }
                    }
                    Err(err) => Err(err),
                }
            }
            Err(err) => {
                let message = format!("{}: {err}", file_path.display());
                Err(ExeErr::new(ExeErrKind::CouldNotReadSourceFile(message)))
            }
        }
    }

    /// Compile AST module node into module object.
    fn compile_module<T: BufRead>(
        &self,
        name: &str,
        source: &mut Source<T>,
    ) -> Result<Module, ExeErr> {
        let ast_module = self.parse_source(source)?;
        let mut compiler = Compiler::new(true);
        let module = compiler.compile_module(name, ast_module).map_err(|err| {
            self.handle_comp_err(&err, source);
            ExeErr::new(ExeErrKind::CompErr(err.kind))
        })?;
        Ok(module)
    }

    /// Parse source text, file, etc into AST module node.
    fn parse_source<T: BufRead>(
        &self,
        source: &mut Source<T>,
    ) -> Result<ast::Module, ExeErr> {
        let scanner = Scanner::new(source);
        let mut parser = Parser::new(scanner);
        let parse_result = parser.parse();
        parse_result.map_err(|err| {
            if let ParseErrKind::ScanErr(scan_err) = err.kind {
                self.handle_scan_err(&scan_err, source);
                ExeErr::new(ExeErrKind::ScanErr(scan_err.kind))
            } else {
                self.handle_parse_err(&err, source);
                ExeErr::new(ExeErrKind::ParseErr(err.kind))
            }
        })
    }

    pub(crate) fn display_stack(&self) {
        eprintln!("{:=<79}", "STACK ");
        self.vm.display_stack();
    }

    fn display_vm_state(&self, result: &VMExeResult) {
        eprintln!("\n{:=<79}", "VM STATE ");
        eprintln!("{:?}", result);
    }

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
                        loc = Location::new(loc.line, loc.col + pos);
                        "Syntax error in format string: expected expression".to_string()
                    }
                    UnmatchedOpeningBracket(pos) => {
                        loc = Location::new(loc.line, loc.col + pos);
                        "Unmatched opening bracket in format string".to_string()
                    }
                    UnmatchedClosingBracket(pos) => {
                        loc = Location::new(loc.line, loc.col + pos);
                        "Unmatched closing bracket in format string".to_string()
                    }
                    ScanErr(_, pos) => {
                        loc = Location::new(loc.line, loc.col + *pos);
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
}
