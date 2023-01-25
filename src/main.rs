use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{parser::ValueSource, ArgMatches};

use feint::cli;
use feint::exe::Executor;
use feint::repl::Repl;
use feint::result::ExeResult;
use feint::vm::{CallDepth, VMState, DEFAULT_MAX_CALL_DEPTH};

/// Interpret a file if one is specified. Otherwise, run the REPL.
fn main() -> ExitCode {
    env_logger::init();

    let app = cli::build_cli();
    let matches = app.get_matches();
    let max_call_depth = *matches.get_one("max_call_depth").unwrap();
    let debug = *matches.get_one::<bool>("debug").unwrap();

    let max_call_depth = match matches.value_source("max_call_depth") {
        Some(ValueSource::DefaultValue) => DEFAULT_MAX_CALL_DEPTH,
        _ => max_call_depth,
    };

    let return_code = match matches.subcommand() {
        Some(("run", matches)) => handle_run(matches, max_call_depth, debug),
        Some(("test", matches)) => handle_test(matches, max_call_depth, debug),
        None => handle_run(&matches, max_call_depth, debug),
        Some((name, _)) => {
            unreachable!("Subcommand not defined: {}", name);
        }
    };

    ExitCode::from(return_code)
}

/// Subcommand: run
fn handle_run(matches: &ArgMatches, max_call_depth: CallDepth, debug: bool) -> u8 {
    let file_name = matches.get_one::<String>("FILE_NAME");
    let code = matches.get_one::<String>("code");
    let dis = *matches.get_one::<bool>("dis").unwrap();
    let history_path = matches.get_one::<String>("history_path");
    let save_repl_history = !matches.get_one::<bool>("no_history").unwrap();
    let mut argv: Vec<String> = matches
        .get_many::<String>("argv")
        .unwrap_or_default()
        .map(|v| v.to_string())
        .collect();

    // When running code via -c, the file_name is actually the first
    // arg in argv.
    if code.is_some() {
        if let Some(arg) = file_name {
            argv.insert(0, arg.to_owned());
        }
    }

    // When running the REPL, use incremental mode. This keeps certain
    // errors from being printed in cases where more input might fix the
    // error.
    let incremental = !(code.is_some() || file_name.is_some());

    let mut exe = Executor::new(max_call_depth, argv, incremental, dis, debug);

    if let Err(err) = exe.bootstrap() {
        return handle_exe_result(Err(err));
    }

    let exe_result = if let Some(code) = code {
        exe.execute_text(code)
    } else if let Some(file_name) = file_name {
        if file_name == "-" {
            exe.execute_stdin()
        } else if let Some(path) = get_script_file_path(file_name) {
            exe.execute_file(path.as_path())
        } else {
            exe.execute_module_as_script(file_name)
        }
    } else {
        let history_path = create_repl_history_file(&save_repl_history, history_path);
        exe.install_sigint_handler();
        let mut repl = Repl::new(history_path, exe);
        repl.run()
    };

    handle_exe_result(exe_result)
}

/// Subcommand: test
fn handle_test(matches: &ArgMatches, max_call_depth: CallDepth, debug: bool) -> u8 {
    let argv: Vec<String> = matches
        .get_many::<String>("argv")
        .unwrap_or_default()
        .map(|v| v.to_string())
        .collect();
    let mut exe = Executor::new(max_call_depth, argv, false, false, debug);
    if let Err(err) = exe.bootstrap() {
        return handle_exe_result(Err(err));
    }
    handle_exe_result(exe.execute_module_as_script("std.test"))
}

// Utilities -----------------------------------------------------------

/// Get script file path from `name`.
///
/// If `name` refers to an existing file path _or_ is absolute _or_ has
/// a `.fi` extension, return `Path` for `name`.
///
/// If `name` is "main", unconditionally return `Path` for main script
/// at `./src/main.fi`. If there's no main script, this will cause an
/// error.
///
/// Otherwise, try to find a script in `./scripts`. If this fails,
/// return `None`.
fn get_script_file_path(name: &String) -> Option<PathBuf> {
    let path = Path::new(name);
    let path = path.to_path_buf();

    if path.is_file()
        || path.is_absolute()
        || path.extension() == Some(OsStr::new("fi"))
    {
        Some(path)
    } else if name == "main" {
        // NOTE: main can only refer to src/main.fi and not a script
        let main_path = Path::new("./src").join("main.fi");
        Some(main_path)
    } else {
        let mut script_path = Path::new("./scripts").join(name);
        script_path.set_extension("fi");
        if script_path.is_file() {
            Some(script_path)
        } else {
            None
        }
    }
}

/// Convert REPL history path from CLI to a `PathBuf`, if possible.
fn create_repl_history_file(cond: &bool, path: Option<&String>) -> Option<PathBuf> {
    if !cond {
        return None;
    }

    let default = String::from("repl-history");
    let path = str_to_path_buf(path, Some(&default));

    path.as_ref()?;

    let path = path.unwrap();

    if path.is_file() {
        return Some(path);
    }

    if path.is_dir() {
        eprintln!("WARNING: REPL history path is a directory: {}", path.display());
        eprintln!("WARNING: REPL history will not be saved");
        eprintln!();
        return None;
    }

    if let Some(parent) = path.parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            eprintln!(
                "WARNING: Could not create REPL history directory: {}",
                parent.display()
            );
            eprintln!("WARNING: {err}");
            eprintln!("WARNING: REPL history will not be saved");
            eprintln!();
            None
        } else {
            eprintln!("INFO: Created REPL history directory: {}", parent.display());
            eprintln!();
            Some(path)
        }
    } else {
        Some(path)
    }
}

fn handle_exe_result(exe_result: ExeResult) -> u8 {
    match exe_result {
        Ok(vm_state) => match vm_state {
            VMState::Running => {
                eprintln!("VM should be idle or halted, not running");
                255
            }
            VMState::Idle(_) => 0,
            VMState::Halted(0) => 0,
            VMState::Halted(code) => code,
        },
        Err(err) => {
            if let Some(exit_code) = err.exit_code() {
                exit_code
            } else {
                eprintln!("{err}");
                255
            }
        }
    }
}

/// Get path for str, expanding leading ~ to user home directory. The
/// default path is used when the input path is None, "", or the home
/// directory isn't found.
fn str_to_path_buf(path: Option<&String>, default: Option<&String>) -> Option<PathBuf> {
    let home = dirs::home_dir();

    let default = if let Some(default) = default {
        str_to_path_buf(Some(default), None)
    } else {
        None
    };

    if let Some(path) = path {
        if path.is_empty() {
            default
        } else if path == "~" {
            if let Some(home) = home {
                Some(home)
            } else {
                default
            }
        } else if path.starts_with('~') {
            if let Some(home) = home {
                Some(home.join(&path[2..]))
            } else {
                default
            }
        } else {
            Some(Path::new(path).to_path_buf())
        }
    } else {
        default
    }
}
