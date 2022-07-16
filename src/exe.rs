//! Front end for executing code from a source on a VM.
use std::io::BufRead;

use crate::compiler::{compile, CompilationErr, CompilationErrKind};
use crate::parser::{ParseErr, ParseErrKind, Parser};
use crate::result::{ExeErr, ExeErrKind, ExeResult};
use crate::scanner::{ScanErr, ScanErrKind, Scanner};
use crate::util::{
    source_from_file, source_from_stdin, source_from_text, Location, Source,
};
use crate::vm::{Inst, RuntimeErr, RuntimeErrKind, VM};

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
    pub fn execute_file(&mut self, file_path: &'a str) -> ExeResult {
        match source_from_file(file_path) {
            Ok(mut source) => {
                self.current_file_name = file_path;
                self.execute_source(&mut source)
            }
            Err(err) => {
                Err(ExeErr::new(ExeErrKind::CouldNotReadSourceFileErr(err.to_string())))
            }
        }
    }

    /// Execute stdin.
    pub fn execute_stdin(&mut self) -> ExeResult {
        self.current_file_name = "<stdin>";
        let mut source = source_from_stdin();
        self.execute_source(&mut source)
    }

    /// Execute text.
    pub fn execute_text(
        &mut self,
        text: &str,
        file_name: Option<&'a str>,
    ) -> ExeResult {
        self.current_file_name = file_name.unwrap_or("<text>");
        let mut source = source_from_text(text);
        self.execute_source(&mut source)
    }

    /// Execute source.
    pub fn execute_source<T: BufRead>(&mut self, source: &mut Source<T>) -> ExeResult {
        let scanner = Scanner::new(source);
        let mut parser = Parser::new(scanner.into_iter());
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
        let chunk = match compile(self.vm, program) {
            Ok(chunk) => chunk,
            Err(err) => {
                self.print_err_line(
                    source.line_no,
                    source.get_current_line().unwrap_or("<none>"),
                );
                self.handle_compilation_err(&err);
                return Err(ExeErr::new(ExeErrKind::CompilationErr(err.kind)));
            }
        };
        self.execute_chunk(chunk)
    }

    /// Execute a chunk (a list of instructions).
    pub fn execute_chunk(&mut self, chunk: Vec<Inst>) -> ExeResult {
        let result = if cfg!(debug_assertions) {
            if self.dis {
                eprintln!("{:=<72}", "INSTRUCTIONS ");
            } else if self.debug {
                eprintln!("{:=<72}", "OUTPUT ");
            }
            self.vm.execute(chunk, self.dis)
        } else if self.dis {
            eprintln!("{:=<72}", "INSTRUCTIONS ");
            let result = self.vm.dis_list(&chunk);
            eprintln!("NOTE: Full disassembly is only available in debug builds");
            result
        } else {
            if self.debug {
                eprintln!("{:=<72}", "OUTPUT ");
            }
            self.vm.execute(chunk, false)
        };
        if self.debug {
            eprintln!("{:=<72}", "STACK ");
            self.vm.display_stack();
            eprintln!("{:=<72}", "VM STATE ");
            eprintln!("{:?}", result);
        }
        match result {
            Ok(vm_state) => Ok(vm_state),
            Err(err) => {
                self.print_err_line(0, "<line not available (TODO)>");
                self.handle_runtime_err(&err);
                Err(ExeErr::new(ExeErrKind::RuntimeErr(err.kind)))
            }
        }
    }

    fn print_err_line(&self, line_no: usize, line: &str) {
        let file_name = self.current_file_name;
        let line = line.trim_end();
        eprintln!("\n  Error in {file_name} on line {line_no}:\n\n    |\n    |{line}");
    }

    fn print_err_message(&self, message: String, loc: Location) {
        if message.len() > 0 {
            let marker_loc = if loc.col == 0 { 0 } else { loc.col - 1 };
            eprintln!("    |{:>marker_loc$}^\n\n  {}\n", "", message);
        }
    }

    fn ignore_scan_err(&self, err: &ScanErr) -> bool {
        use ScanErrKind::*;
        self.incremental
            && match &err.kind {
                UnmatchedOpeningBracket(_) | UnterminatedString(_) => true,
                _ => false,
            }
    }

    fn handle_scan_err(&self, err: &ScanErr) {
        use ScanErrKind::*;
        let mut loc = err.location.clone();
        let col = loc.col;
        let message = match &err.kind {
            UnexpectedCharacter(c) => {
                format!("Syntax error: Unexpected character at column {}: '{}'", col, c)
            }
            UnmatchedOpeningBracket(_) => {
                format!("Unmatched open bracket at {loc}")
            }
            UnterminatedString(_) => {
                format!("Syntax error: Unterminated string literal at {loc}")
            }
            InvalidIndent(num_spaces) => {
                format!("Syntax error: Invalid indent with {num_spaces} spaces (should be a multiple of 4)")
            }
            UnexpectedIndent(_) => {
                format!("Syntax error: Unexpected indent")
            }
            WhitespaceAfterIndent | UnexpectedWhitespace => {
                format!("Syntax error: Unexpected whitespace")
            }
            FormatStringErr(err) => {
                use crate::format::FormatStringErr::*;
                match err {
                    EmptyExpr(pos) => {
                        loc = Location::new(loc.line, loc.col + 2 + pos);
                        format!("Syntax error: expected expression")
                    }
                    _ => {
                        format!("Unhandled format string error at {loc}")
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
        self.incremental
            && match &err.kind {
                ExpectedBlock(_) => true,
                _ => false,
            }
    }

    fn handle_parse_err(&self, err: &ParseErr) {
        use ParseErrKind::*;
        let (loc, message) = match &err.kind {
            ScanErr(_) => {
                unreachable!("Handle ScanErr before calling handle_parse_err");
            }
            UnexpectedToken(token) => {
                let loc = token.start.clone();
                let token = &token.token;
                (loc, format!("Parse error: Unhandled token at {loc}: {token:?}"))
            }
            ExpectedBlock(loc) => {
                (loc.clone(), format!("Parse error: expected indented block at {loc}"))
            }
            ExpectedToken(loc, token) => {
                (loc.clone(), format!("Parse error: expected token '{token}' at {loc}"))
            }
            ExpectedExpr(loc) => {
                (loc.clone(), format!("Parse error: expected expression at {loc}",))
            }
            kind => (Location::new(0, 0), format!("Unhandled parse error: {:?}", kind)),
        };
        self.print_err_message(message, loc);
    }

    fn handle_compilation_err(&self, err: &CompilationErr) {
        let message = match &err.kind {
            CompilationErrKind::VisitErr(message) => {
                format!("Visitation failed: {message}")
            }
        };
        eprintln!("    |\n\n  {}", message);
    }

    fn handle_runtime_err(&self, err: &RuntimeErr) {
        let message = match &err.kind {
            RuntimeErrKind::NameErr(message) => format!("Name error: {message}"),
            RuntimeErrKind::TypeErr(message) => format!("Type error: {message}"),
            kind => format!("Unhandled runtime error: {:?}", kind),
        };
        eprintln!("    |\n\n  {}", message);
    }
}
