use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::builder::FalseyValueParser;
use clap::{parser::ValueSource, value_parser, Arg, ArgAction, ArgMatches, Command};

use feint::exe::Executor;
use feint::repl::Repl;
use feint::result::ExeErrKind;
use feint::vm::{CallDepth, VMState, DEFAULT_MAX_CALL_DEPTH};

/// Interpret a file if one is specified. Otherwise, run the REPL.
fn main() -> ExitCode {
    env_logger::init();

    let file_name_help = concat!(
        "Script to run. Can be:\n\n",
        "1. a path to a script file\n",
        "2. the name of a script in ./scripts (without .fi extension)\n",
        "3. a single dash to read from stdin\n",
    );
    let file_name_arg =
        Arg::new("FILE_NAME").index(1).required(false).help(file_name_help);

    let code_arg = Arg::new("code")
        .short('c')
        .long("code")
        .required(false)
        .num_args(1)
        .help("Use this to run short snippets of code");

    let dis_arg = Arg::new("dis")
        .short('i')
        .long("dis")
        .action(ArgAction::SetTrue)
        .help("disassemble instructions?");

    let history_path_arg = Arg::new("history_path")
        .long("history-path")
        .required(false)
        .num_args(1)
        .default_value("~/.config/feint/repl-history")
        .help("Path to REPL history file");

    let no_history_arg = Arg::new("no_history")
        .long("no-history")
        .action(ArgAction::SetTrue)
        .help("Disable REPL history? [default: history enabled]");

    let argv_help = concat!(
        "Additional args will be set as system.argv.\n",
        "Can be used when running a script and with -c.\n",
        "CANNOT be used when running REPL."
    );
    let argv_arg =
        Arg::new("argv").index(2).trailing_var_arg(true).num_args(0..).help(argv_help);

    let app = Command::new("FeInt")
        .version("0.0.0")
        .arg(
            Arg::new("builtin_module_search_path")
                .short('b')
                .long("builtin-module-search-path")
                .required(false)
                .env("FEINT_BUILTIN_MODULE_SEARCH_PATH")
                .help("Search path for builtin modules"),
        )
        .arg(
            Arg::new("max_call_depth")
                .short('x')
                .long("max-call-depth")
                .default_value("0")
                .value_parser(value_parser!(CallDepth))
                .env("FEINT_MAX_CALL_DEPTH")
                .help("Maximum call/recursion depth"),
        )
        .arg(
            Arg::new("debug")
                .short('d')
                .long("debug")
                .action(ArgAction::SetTrue)
                .value_parser(FalseyValueParser::new())
                .env("FEINT_DEBUG")
                .help("Enable debug mode?"),
        )
        // Subcommand: run (when no subcommand is specified)
        .arg(&file_name_arg)
        .arg(&code_arg)
        .arg(&dis_arg)
        .arg(&history_path_arg)
        .arg(&no_history_arg)
        .arg(&argv_arg)
        .subcommands([
            // Subcommand: run
            Command::new("run")
                .about("Run script or code")
                .arg(&file_name_arg)
                .arg(&code_arg)
                .arg(&dis_arg)
                .arg(&history_path_arg)
                .arg(&no_history_arg)
                .arg(&argv_arg),
            // Subcommand: test
            Command::new("test").about("Run test").arg(
                Arg::new("what")
                    .short('w')
                    .long("what")
                    .action(ArgAction::SetTrue)
                    .help("Specify what to test"),
            ),
        ]);

    let matches = app.get_matches();
    let builtin_module_search_path =
        matches.get_one::<String>("builtin_module_search_path");
    let max_call_depth = *matches.get_one("max_call_depth").unwrap();
    let debug = *matches.get_one::<bool>("debug").unwrap();

    let max_call_depth = match matches.value_source("max_call_depth") {
        Some(ValueSource::DefaultValue) => DEFAULT_MAX_CALL_DEPTH,
        _ => max_call_depth,
    };

    let return_code = match matches.subcommand() {
        Some(("run", matches)) => {
            handle_run(matches, builtin_module_search_path, max_call_depth, debug)
        }
        Some(("test", matches)) => handle_test(matches, max_call_depth, debug),
        None => handle_run(&matches, builtin_module_search_path, max_call_depth, debug),
        Some((name, _)) => {
            unreachable!("Subcommand not defined: {}", name);
        }
    };

    ExitCode::from(return_code)
}

/// Subcommand: run
fn handle_run(
    matches: &ArgMatches,
    builtin_module_search_path: Option<&String>,
    max_call_depth: CallDepth,
    debug: bool,
) -> u8 {
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

    let mut exe = Executor::new(
        builtin_module_search_path.map(|x| x.to_owned()),
        max_call_depth,
        argv,
        false,
        dis,
        debug,
    );

    // XXX: Stop clippy from erroneously suggesting `exe.bootstrap()?`.
    #[allow(clippy::question_mark)]
    let exe_result = if let Err(err) = exe.bootstrap() {
        Err(err)
    } else if let Some(code) = code {
        exe.execute_text(code)
    } else if let Some(file_name) = file_name {
        if file_name == "-" {
            exe.execute_stdin()
        } else {
            let path = get_script_file_path(file_name);
            exe.execute_file(path.as_path())
        }
    } else {
        let history_path = create_repl_history_file(&save_repl_history, history_path);
        exe.install_sigint_handler();
        let mut repl = Repl::new(history_path, exe);
        repl.run()
    };

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
            let exit_code = err.exit_code().unwrap_or(255);
            match err.kind {
                ExeErrKind::Bootstrap(message) => eprintln!("{message}"),
                ExeErrKind::CouldNotReadSourceFile(message) => eprintln!("{message}"),
                ExeErrKind::ReplErr(message) => eprintln!("{message}"),
                _ => (),
            }
            exit_code
        }
    }
}

/// Subcommand: test
fn handle_test(_matches: &ArgMatches, _max_call_depth: CallDepth, _debug: bool) -> u8 {
    println!("Command test not yet implemented");
    0
}

// Utilities -----------------------------------------------------------

/// Get script file path from `name`. If `name` refers to an existing
/// file path _or_ is absolute _or_ has an extension, return a path
/// object for `name`.
///
/// Otherwise, try to find a script in `./scripts`. If this fails,
/// return a path object for `name`.
fn get_script_file_path(name: &String) -> PathBuf {
    let path = Path::new(name);
    let path = path.to_path_buf();

    if path.is_file() || path.is_absolute() || path.extension().is_some() {
        return path;
    }

    let mut script_path = Path::new("./scripts").join(&path);
    script_path.set_extension("fi");

    if script_path.is_file() {
        script_path
    } else {
        path
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
