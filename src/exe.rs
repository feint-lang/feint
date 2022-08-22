//! Front end for executing code from a source on a VM.
use std::io::BufRead;

use crate::compiler::{CompErr, CompErrKind, Compiler};
use crate::dis;
use crate::modules;
use crate::parser::{ParseErr, ParseErrKind, Parser};
use crate::result::{ExeErr, ExeErrKind, ExeResult};
use crate::scanner::{ScanErr, ScanErrKind, Scanner, Token, TokenWithLocation};
use crate::types::{Module, ObjectRef};
use crate::util::{
    source_from_file, source_from_stdin, source_from_text, Location, Source,
};
use crate::vm::{Code, RuntimeErr, RuntimeErrKind, VMExeResult, VMState, VM};

pub struct Executor<'a> {
    pub vm: &'a mut VM,
    incremental: bool,
    dis: bool,
    debug: bool,
    current_file_name: &'a str,
}

impl<'a> Executor<'a> {
    pub fn new(vm: &'a mut VM, incremental: bool, dis: bool, debug: bool) -> Self {
        Self { vm, incremental, dis, debug, current_file_name: "<none>" }
    }

    pub fn default(vm: &'a mut VM) -> Self {
        Self {
            vm,
            incremental: false,
            dis: false,
            debug: false,
            current_file_name: "<default>",
        }
    }

    /// Execute source from file.
    pub fn execute_file(&mut self, file_path: &'a str, argv: Vec<&str>) -> ExeResult {
        match source_from_file(file_path) {
            Ok(mut source) => {
                self.current_file_name = file_path;
                self.execute_source(&mut source, argv)
            }
            Err(err) => {
                let message = format!("{file_path}: {err}");
                Err(ExeErr::new(ExeErrKind::CouldNotReadSourceFile(message)))
            }
        }
    }

    /// Execute stdin.
    pub fn execute_stdin(&mut self) -> ExeResult {
        self.current_file_name = "<stdin>";
        let mut source = source_from_stdin();
        self.execute_source(&mut source, vec![])
    }

    /// Execute text.
    pub fn execute_text(&mut self, text: &str) -> ExeResult {
        self.current_file_name = "<text>";
        let mut source = source_from_text(text);
        self.execute_source(&mut source, vec![])
    }

    /// Execute source.
    pub fn execute_source<T: BufRead>(
        &mut self,
        source: &mut Source<T>,
        argv: Vec<&str>,
    ) -> ExeResult {
        let code = self.compile_source(source, argv)?;
        if self.dis {
            let mut disassembler = dis::Disassembler::new();
            disassembler.disassemble(&code);
            if self.debug {
                println!();
                self.display_stack();
            }
            Ok(VMState::Halted(0))
        } else {
            if let Err(err) = modules::add_system_module_to_system() {
                return Err(ExeErr::new(ExeErrKind::RuntimeErr(err.kind)));
            }
            match modules::add_module("$main", code) {
                Ok(main_module) => {
                    let mut main_module = main_module.write().unwrap();
                    let main_module = main_module.down_to_mod_mut().unwrap();
                    self.execute_module(main_module, self.debug, source)
                }
                Err(err) => Err(ExeErr::new(ExeErrKind::RuntimeErr(err.kind))),
            }
        }
    }

    pub fn execute_repl(&mut self, text: &str, module: ObjectRef) -> ExeResult {
        self.current_file_name = "<repl>";
        let source = &mut source_from_text(text);
        let code = self.compile_source(source, vec![])?;
        let mut module = module.write().unwrap();
        let module = module.down_to_mod_mut().unwrap();
        module.set_code(code);
        let result = self.execute_module(module, self.debug, source);

        // Add globals to REPL module namespace.
        // TODO: Find a better way to do this.
        for (name, val) in self.vm.ctx.iter_vars() {
            module.add_global(name, val.clone());
        }

        result
    }

    /// Execute a code (a list of instructions).
    pub fn execute_module<T: BufRead>(
        &mut self,
        module: &mut Module,
        debug: bool,
        source: &mut Source<T>,
    ) -> ExeResult {
        let result = self.vm.execute(module.code());
        if debug {
            self.display_stack();
            self.display_vm_state(&result);
        }
        result.map_err(|err| {
            let start = self.vm.loc().0;
            let line =
                source.get_line(start.line).unwrap_or("<source line not available>");
            self.print_err_line(start.line, line);
            self.handle_runtime_err(&err);
            ExeErr::new(ExeErrKind::RuntimeErr(err.kind))
        })
    }

    /// Compile source into a `Code` object.
    fn compile_source<T: BufRead>(
        &self,
        source: &mut Source<T>,
        argv: Vec<&str>,
    ) -> Result<Code, ExeErr> {
        let scanner = Scanner::new(source);
        let mut parser = Parser::new(scanner);
        let program = match parser.parse() {
            Ok(program) => program,
            Err(err) => {
                return if let ParseErrKind::ScanErr(scan_err) = err.kind {
                    if !self.ignore_scan_err(&scan_err) {
                        self.print_err_line(
                            source.line_no,
                            source.get_current_line().unwrap_or("<none>"),
                        );
                        self.handle_scan_err(&scan_err);
                    }
                    Err(ExeErr::new(ExeErrKind::ScanErr(scan_err.kind)))
                } else {
                    if !self.ignore_parse_err(&err) {
                        let loc = err.loc();
                        self.print_err_line(
                            loc.line,
                            source.get_line(loc.line).unwrap_or("<none>"),
                        );
                        self.handle_parse_err(&err);
                    }
                    Err(ExeErr::new(ExeErrKind::ParseErr(err.kind)))
                };
            }
        };
        let mut compiler = Compiler::new();
        let code = match compiler.compile_script(program, argv) {
            Ok(code) => code,
            Err(err) => {
                if !self.ignore_comp_err(&err) {
                    let (start, _end) = err.loc();
                    self.print_err_line(
                        start.line,
                        source.get_line(start.line).unwrap_or("<none>"),
                    );
                    self.handle_comp_err(&err);
                }
                return Err(ExeErr::new(ExeErrKind::CompErr(err.kind)));
            }
        };
        Ok(code)
    }

    fn display_stack(&self) {
        eprintln!("{:=<79}", "STACK ");
        self.vm.display_stack();
    }

    fn display_vm_state(&self, result: &VMExeResult) {
        eprintln!("\n{:=<79}", "VM STATE ");
        eprintln!("{:?}", result);
    }

    fn print_err_line(&self, line_no: usize, line: &str) {
        let file_name = self.current_file_name;
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

    fn ignore_scan_err(&self, err: &ScanErr) -> bool {
        use ScanErrKind::*;
        self.incremental
            && matches!(
                &err.kind,
                ExpectedBlock
                    | ExpectedIndentedBlock(_)
                    | UnmatchedOpeningBracket(_)
                    | UnterminatedStr(_)
            )
    }

    fn handle_scan_err(&self, err: &ScanErr) {
        use ScanErrKind::*;
        let mut loc = err.location;
        let col = loc.col;
        let message = match &err.kind {
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
        self.print_err_message(message, loc, loc);
    }

    fn ignore_parse_err(&self, err: &ParseErr) -> bool {
        use ParseErrKind::*;
        self.incremental && matches!(&err.kind, ExpectedBlock(_))
    }

    fn handle_parse_err(&self, err: &ParseErr) {
        use ParseErrKind::*;
        let loc = err.loc();
        let message = match &err.kind {
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
        self.print_err_message(message, loc, loc);
    }

    fn handle_comp_err(&self, err: &CompErr) {
        use CompErrKind::*;
        let (start, end) = err.loc();
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
            GlobalNotFound(name, ..) => {
                format!("global var not found: {name}")
            }
            VarArgsMustBeLast(..) => {
                "var args must be last in parameter list".to_owned()
            }
        };
        let message = format!("Compilation error: {message}");
        self.print_err_message(message, start, end);
    }

    fn ignore_comp_err(&self, err: &CompErr) -> bool {
        use CompErrKind::*;
        self.incremental && matches!(&err.kind, LabelNotFoundInScope(..))
    }

    fn handle_runtime_err(&self, err: &RuntimeErr) {
        use RuntimeErrKind::*;
        let (start, end) = self.vm.loc();
        let message = match &err.kind {
            AssertionFailed(message) => {
                if message.is_empty() {
                    "Assertion failed".to_string()
                } else {
                    format!("Assertion failed with message: {message}")
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
            AttrDoesNotExist(type_name, name) => {
                format!("Attribute does not exist on type {type_name}: {name}")
            }
            AttrCannotBeSet(type_name, name) => {
                format!("Attribute cannot be set on {type_name}: {name}")
            }
            ItemDoesNotExist(type_name, index) => {
                format!("Item with index does not exist on type {type_name}: {index}")
            }
            ItemCannotBeSet(type_name, index) => {
                format!("Item cannot be set by index on {type_name}: {index}")
            }
            IndexOutOfBounds(type_name, index) => {
                format!("Index out of bounds on {type_name}: {index}")
            }
            NotCallable(type_name) => format!("Object is not callable: {type_name}"),
            kind => format!("Unhandled runtime error: {:?}", kind),
        };
        self.print_err_message(message, start, end);
    }
}
