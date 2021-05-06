/// Run REPL until user exits.
use std::path::{Path, PathBuf};

use dirs;

use rustyline::error::ReadlineError;
use rustyline::validate::ValidationResult::Incomplete;

use crate::instructions::Instruction;
use crate::parser::Parser;
use crate::scanner::Scanner;
use crate::tokens::{Token, TokenWithPosition};
use crate::vm::{VMState, VM};

type ExitData = (i32, String);
type ExitResult = Result<Option<String>, ExitData>;

pub struct Runner<'a> {
    reader: rustyline::Editor<()>,
    history_path: Option<&'a Path>,
    vm: VM<'a>,
    debug: bool,
}

impl<'a> Runner<'a> {
    pub fn new(history_path: Option<&'a Path>, debug: bool) -> Runner<'a> {
        Runner {
            reader: rustyline::Editor::<()>::new(),
            history_path,
            vm: VM::new(),
            debug,
        }
    }

    /// Get the default history path, which is either ~/.feint_history
    /// or, if the user's home directory can't be located,
    /// ./.feint_history.
    pub fn default_history_path() -> PathBuf {
        let home = dirs::home_dir();
        let base_path = home.unwrap_or_default();
        let history_path_buf = base_path.join(".feint_history");
        history_path_buf
    }

    pub fn run(&mut self) -> ExitResult {
        println!("Welcome to the FeInt REPL (read/eval/print loop)");
        println!("Type a line of code, then hit Enter to evaluate it");
        println!("Type .exit or .quit to exit");

        self.load_history();

        loop {
            match self.read_line("→ ", true) {
                Ok(None) => (),
                Ok(Some(input)) => {
                    // Evaluate the input. If eval returns a result of
                    // any kind (ok or err), exit the loop and shut down
                    // the REPL.
                    match self.eval(input.as_str()) {
                        Some(result) => {
                            self.vm.halt();
                            return result;
                        }
                        None => (),
                    }
                }
                // User hit Ctrl-C or Ctrl-D.
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                    self.vm.halt();
                    return Ok(None);
                }
                // Unexpected error encountered while attempting to read
                // a line.
                Err(err) => {
                    self.vm.halt();
                    return Err((1, format!("Could not read line: {}", err)));
                }
            }
        }
    }

    /// Get a line of input from the user.
    fn read_line(&mut self, prompt: &str, trim: bool) -> Result<Option<String>, ReadlineError> {
        match self.reader.readline(prompt) {
            Ok(input) if input.trim().len() == 0 => match trim {
                true => Ok(None),
                false => Ok(Some(input)),
            },
            Ok(input) => Ok(Some(input)),
            Err(err) => Err(err),
        }
    }

    pub fn eval(&mut self, source: &str) -> Option<ExitResult> {
        let mut scanner = Scanner::new();

        let instructions = match source.trim() {
            ".exit" | ".halt" | ".quit" => {
                vec![Instruction::Halt(0)]
            }
            _ => match scanner.scan(source) {
                Ok(tokens) => {
                    self.add_history_entry(source);
                    self.parse(tokens);
                    let mut instructions: Vec<Instruction> = vec![];
                    instructions
                }
                Err((error_token, tokens)) => match error_token.token {
                    Token::Unknown(c) => {
                        self.add_history_entry(source);
                        let col_no = error_token.col_no;
                        eprintln!("{: >width$}^", "", width = col_no + 1);
                        eprintln!("Syntax error: unknown token at column {}: {}", col_no, c);
                        return None;
                    }
                    Token::UnterminatedString(string) => {
                        self.add_history_entry(source);
                        self.parse(tokens);
                        loop {
                            return match self.read_line("+ ", false) {
                                Ok(None) => {
                                    // Blank line (can't happen?)
                                    let input = string + "\n";
                                    self.eval(input.as_str())
                                }
                                Ok(Some(new_input)) => {
                                    let input = string + "\n" + new_input.as_str();
                                    self.eval(input.as_str())
                                }
                                Err(err) => Some(Err((1, format!("{}", err)))),
                            };
                        }
                    }
                    token => {
                        // This shouldn't happen.
                        return Some(Err((1, format!("{:?}", token))));
                    }
                },
            },
        };

        match self.vm.execute(&instructions) {
            VMState::Halted(0, option_message) => Some(Ok(option_message)),
            VMState::Halted(code, Some(message)) => Some(Err((code, message))),
            VMState::Halted(code, None) => Some(Err((code, "Unknown Error".to_string()))),
            VMState::Idle => None,
        }
    }

    fn parse(&self, tokens: Vec<TokenWithPosition>) {
        if self.debug {
            for t in tokens.iter() {
                eprintln!("{:?}", t);
            }
        }
        let init_tokens: Vec<TokenWithPosition> = vec![];
        let mut parser = Parser::new(&init_tokens);
        let result = parser.parse(&tokens);
        println!("{}", result);
    }

    fn load_history(&mut self) {
        match self.history_path {
            Some(path) => {
                println!("REPL history will be saved to {}", path.to_string_lossy());
                match self.reader.load_history(path) {
                    Ok(_) => (),
                    Err(err) => eprintln!("Could not load REPL history: {}", err),
                }
            }
            None => (),
        }
    }

    fn add_history_entry(&mut self, input: &str) {
        match self.history_path {
            Some(path) => {
                self.reader.add_history_entry(input);
                match self.reader.save_history(path) {
                    Ok(_) => (),
                    Err(err) => eprintln!("Could not save REPL history: {}", err),
                }
            }
            None => (),
        }
    }
}
