/// Run REPL until user exits.
use std::path::{Path, PathBuf};

use dirs;
use rustyline::error::ReadlineError;

use crate::parser::parse;
use crate::scanner::{scan, ScanError, ScanErrorType, TokenWithLocation};
use crate::vm::Instruction;
use crate::vm::Namespace;
use crate::vm::{VMState, VM};

type ExitData = (i32, String);
type ExitResult = Result<Option<String>, ExitData>;

pub fn run(debug: bool) -> ExitResult {
    let history_path = Runner::default_history_path();
    let namespace = Namespace::default();
    let vm = VM::new(namespace);
    let mut runner = Runner::new(Some(history_path.as_path()), vm, debug);
    runner.run()
}

pub struct Runner<'a> {
    reader: rustyline::Editor<()>,
    history_path: Option<&'a Path>,
    vm: VM<'a>,
    debug: bool,
}

impl<'a> Runner<'a> {
    pub fn new(history_path: Option<&'a Path>, vm: VM<'a>, debug: bool) -> Self {
        Runner { reader: rustyline::Editor::<()>::new(), history_path, vm, debug }
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
                Ok(None) => {
                    // Blank or all-whitespace line.
                    ()
                }
                Ok(Some(input)) => {
                    // Evaluate the input. If eval returns a result of
                    // any kind (ok or err), exit the loop and shut down
                    // the REPL.
                    match self.eval(input.as_str()) {
                        Some(result) => {
                            self.vm.halt();
                            break result;
                        }
                        None => (),
                    }
                }
                // User hit Ctrl-C or Ctrl-D.
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                    self.vm.halt();
                    break Ok(None);
                }
                // Unexpected error encountered while attempting to read
                // a line.
                Err(err) => {
                    self.vm.halt();
                    break Err((1, format!("Could not read line: {}", err)));
                }
            }
        }
    }

    /// Get a line of input from the user. If the line comprises only
    /// whitespace *and* ``trim_blank`` is set, the line will be trimmed
    /// and ``None`` will be returned.
    fn read_line(
        &mut self,
        prompt: &str,
        trim_blank: bool,
    ) -> Result<Option<String>, ReadlineError> {
        match self.reader.readline(prompt) {
            Ok(input) if trim_blank && input.trim().len() == 0 => Ok(None),
            Ok(input) => Ok(Some(input)),
            Err(err) => Err(err),
        }
    }

    pub fn eval(&mut self, source: &str) -> Option<ExitResult> {
        let instructions: Vec<Instruction> = match source.trim() {
            ".exit" | ".halt" | ".quit" => {
                vec![Instruction::Halt(0)]
            }
            _ => match scan(source) {
                Ok(tokens) => {
                    self.add_history_entry(source);
                    self.parse(tokens)
                }
                Err(err) => match err {
                    ScanError { error: ScanErrorType::UnknownToken(c), location } => {
                        self.add_history_entry(source);
                        let col = location.col;
                        eprintln!("{: >width$}^", "", width = col + 1);
                        eprintln!(
                            "Syntax error: unknown token at column {}: '{}'",
                            col, c
                        );
                        return None;
                    }
                    ScanError {
                        error: ScanErrorType::UnterminatedString(_),
                        location: _,
                    } => loop {
                        return match self.read_line("+ ", false) {
                            Ok(None) => {
                                let input = source.to_string() + "\n";
                                self.eval(input.as_str())
                            }
                            Ok(Some(new_input)) => {
                                let input =
                                    source.to_string() + "\n" + new_input.as_str();
                                self.eval(input.as_str())
                            }
                            Err(err) => Some(Err((1, format!("{}", err)))),
                        };
                    },
                    ScanError {
                        error: ScanErrorType::WhitespaceAfterIndent,
                        location,
                    }
                    | ScanError {
                        error: ScanErrorType::UnexpectedWhitespace,
                        location,
                    } => {
                        self.add_history_entry(source);
                        let col_no = location.col;
                        eprintln!("{: >width$}^", "", width = col_no + 1);
                        eprintln!("Syntax error: unexpected whitespace");
                        return None;
                    }
                    err => {
                        return Some(Err((
                            1,
                            format!("Unhandled scan error: {:?}", err),
                        )));
                    }
                },
            },
        };

        match self.vm.execute(&instructions) {
            VMState::Halted(0, option_message) => Some(Ok(option_message)),
            VMState::Halted(code, Some(message)) => Some(Err((code, message))),
            VMState::Halted(code, None) => {
                Some(Err((code, "Unknown Error".to_string())))
            }
            VMState::Idle => None,
        }
    }

    fn parse(&self, tokens: Vec<TokenWithLocation>) -> Vec<Instruction> {
        if self.debug {
            for t in tokens.iter() {
                eprintln!("{}", t);
            }
        }
        // let ast = parse(&tokens);
        // eprintln!("{:?}", ast);

        let mut instructions = vec![];
        instructions
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
