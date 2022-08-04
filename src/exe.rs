//! Front end for executing code from a source on a VM.
use std::io::BufRead;

use crate::compiler::{compile, CompErr, CompErrKind};
use crate::parser::{ParseErr, ParseErrKind, Parser};
use crate::result::{ExeErr, ExeErrKind, ExeResult};
use crate::scanner::{ScanErr, ScanErrKind, Scanner, Token};
use crate::util::{
    source_from_file, source_from_stdin, source_from_text, Location, Source,
};
use crate::vm::{Code, RuntimeErr, RuntimeErrKind, VM};

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
    pub fn execute_text(
        &mut self,
        text: &str,
        file_name: Option<&'a str>,
    ) -> ExeResult {
        self.current_file_name = file_name.unwrap_or("<text>");
        let mut source = source_from_text(text);
        self.execute_source(&mut source, vec![])
    }

    /// Execute source.
    pub fn execute_source<T: BufRead>(
        &mut self,
        source: &mut Source<T>,
        argv: Vec<&str>,
    ) -> ExeResult {
        let scanner = Scanner::new(source);
        let mut parser = Parser::new(scanner);
        let program = match parser.parse() {
            Ok(program) => program,
            Err(err) => {
                return match err.kind {
                    ParseErrKind::ScanErr(scan_err) => {
                        if !self.ignore_scan_err(&scan_err) {
                            self.print_err_line(
                                source.line_no,
                                source.get_current_line().unwrap_or("<none>"),
                            );
                            self.handle_scan_err(&scan_err);
                        }
                        Err(ExeErr::new(ExeErrKind::ScanErr(scan_err.kind)))
                    }
                    _ => {
                        if !self.ignore_parse_err(&err) {
                            self.print_err_line(
                                source.line_no,
                                source.get_current_line().unwrap_or("<none>"),
                            );
                            self.handle_parse_err(&err);
                        }
                        Err(ExeErr::new(ExeErrKind::ParseErr(err.kind)))
                    }
                };
            }
        };
        let code = match compile(program, argv) {
            Ok(code) => code,
            Err(err) => {
                if !self.ignore_comp_err(&err) {
                    self.print_err_line(
                        source.line_no,
                        source.get_current_line().unwrap_or("<none>"),
                    );
                    self.handle_comp_err(&err);
                }
                return Err(ExeErr::new(ExeErrKind::CompErr(err.kind)));
            }
        };
        self.execute_code(code)
    }

    /// Execute a code (a list of instructions).
    pub fn execute_code(&mut self, code: Code) -> ExeResult {
        if self.dis {
            eprintln!("{:=<79}", "INSTRUCTIONS ");
        } else if self.debug {
            eprintln!("{:=<79}", "OUTPUT ");
        }

        let result = self.vm.execute(&code, self.dis);

        let num_funcs = if self.dis {
            eprintln!();
            self.vm.dis_functions(&code)
        } else {
            0
        };

        if self.debug {
            if !self.dis || num_funcs > 0 {
                eprintln!();
            }
            eprintln!("{:=<79}", "STACK ");
            self.vm.display_stack();
            eprintln!("\n{:=<79}", "CONSTANTS ");
            self.vm.display_constants();
            eprintln!("\n{:=<79}", "VARS ");
            self.vm.display_vars();
            eprintln!("\n{:=<79}", "VM STATE ");
            eprintln!("{:?}", result);
        }

        result.map_err(|err| {
            self.print_err_line(0, "<line not available (TODO)>");
            self.handle_runtime_err(&err);
            ExeErr::new(ExeErrKind::RuntimeErr(err.kind))
        })
    }

    fn print_err_line(&self, line_no: usize, line: &str) {
        let file_name = self.current_file_name;
        let line = line.trim_end();
        eprintln!("\n  Error in {file_name} on line {line_no}:\n\n    |\n    |{line}");
    }

    fn print_err_message(&self, message: String, loc: Location) {
        if !message.is_empty() {
            let marker_loc = if loc.col == 0 { 0 } else { loc.col - 1 };
            eprintln!("    |{:>marker_loc$}^\n\n  {}\n", "", message);
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
        self.print_err_message(message, loc);
    }

    fn ignore_parse_err(&self, err: &ParseErr) -> bool {
        use ParseErrKind::*;
        self.incremental && matches!(&err.kind, ExpectedBlock(_))
    }

    fn handle_parse_err(&self, err: &ParseErr) {
        use ParseErrKind::*;
        let (loc, message) = match &err.kind {
            ScanErr(_) => {
                unreachable!("Handle ScanErr before calling handle_parse_err");
            }
            UnexpectedToken(token) => {
                let loc = token.start;
                let token = &token.token;
                if token == &Token::EndOfStatement {
                    (loc, format!("Syntax error at {loc} (unexpected end of statement)"))
                } else {
                    (loc, format!("Parse error: unexpected token at {loc}: {token:?}"))
                }
            }
            ExpectedBlock(loc) => {
                (*loc, format!("Parse error: expected indented block at {loc}"))
            }
            ExpectedToken(loc, token) => {
                (*loc, format!("Parse error: expected token '{token}' at {loc}"))
            }
            ExpectedExpr(loc) => {
                (*loc, format!("Parse error: expected expression at {loc}",))
            }
            ExpectedIdent(loc) => {
                (*loc, format!("Parse error: expected identifier at {loc}",))
            }
            UnexpectedBreak(loc) => (
                *loc,
                format!(
                    "Parse error: unexpected break at {loc} (break must be in a loop)"
                ),
            ),
            UnexpectedContinue(loc) => (
                *loc,
                format!(
                    "Parse error: unexpected continue at {loc} (continue must be in a loop)"
                ),
            ),
            InlineMatchNotAllowed(loc) => (
                *loc, "Parse error: match blocks must be idented".to_string(),
            ),
            MatchDefaultMustBeLast(loc) => (
                *loc, "Parse error: extra match arm found after default match arm".to_string(),
            ),
            SyntaxErr(loc) => (*loc, format!("Syntax error at {loc}",)),
            kind => (Location::new(0, 0), format!("Unhandled parse error: {:?}", kind)),
        };
        self.print_err_message(message, loc);
    }

    fn handle_comp_err(&self, err: &CompErr) {
        use CompErrKind::*;
        let message = match &err.kind {
            UnhandledExpr(start, end) => {
                format!("unhandled expression at {start} -> {end}")
            }
            LabelNotFoundInScope(name) => {
                format!("label not found in scope: {name}")
            }
            CannotJumpOutOfFunc(name) => {
                format!("Cannot jump out of function: label {name} not found or defined in outer scope")
            }
            DuplicateLabelInScope(name) => {
                format!("duplicate label in scope: {name}")
            }
            ExpectedIdent => "expected identifier".to_string(),
            CannotAssignSpecialIdent(name) => {
                format!("cannot assign to special name: {name}")
            }
        };
        eprintln!("    |\n\n  Compilation error: {}", message);
    }

    fn ignore_comp_err(&self, err: &CompErr) -> bool {
        use CompErrKind::*;
        self.incremental && matches!(&err.kind, LabelNotFoundInScope(_))
    }

    fn handle_runtime_err(&self, err: &RuntimeErr) {
        use RuntimeErrKind::*;
        let message = match &err.kind {
            RecursionDepthExceeded(max_call_depth) => {
                format!("Maximum recursion depth of {max_call_depth} was exceeded")
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
        eprintln!("    |\n\n  {}", message);
    }
}
