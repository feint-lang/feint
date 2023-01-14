use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::builder::FalseyValueParser;
use clap::{parser::ValueSource, value_parser, Arg, ArgAction, ArgMatches, Command};

use feint::config::CONFIG;
use feint::repl;
use feint::run;
use feint::vm::{CallDepth, RuntimeErr, DEFAULT_MAX_CALL_DEPTH};

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
                .default_value("./src/modules")
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
        matches.get_one::<String>("builtin_module_search_path").unwrap();
    let max_call_depth = *matches.get_one("max_call_depth").unwrap();
    let dis = *matches.get_one::<bool>("dis").unwrap();
    let debug = *matches.get_one::<bool>("debug").unwrap();

    let max_call_depth = match matches.value_source("max_call_depth") {
        Some(ValueSource::DefaultValue) => DEFAULT_MAX_CALL_DEPTH,
        _ => max_call_depth,
    };

    let mut config = CONFIG.write().unwrap();

    let config_results = vec![
        config.set_str("builtin_module_search_path", builtin_module_search_path),
        config.set_usize("max_call_depth", max_call_depth),
        config.set_bool("debug", debug),
    ];

    if let Some((exit_code, messages)) = check_config_results(config_results) {
        eprintln!("Aborting due to invalid config:\n");
        for message in messages {
            eprintln!("{message}")
        }
        return ExitCode::from(exit_code);
    }

    drop(config);

    let return_code = match matches.subcommand() {
        Some(("run", matches)) => handle_run(matches, max_call_depth, dis, debug),
        Some(("test", matches)) => handle_test(matches, max_call_depth, dis, debug),
        None => handle_run(&matches, max_call_depth, dis, debug),
        Some((name, _)) => {
            unreachable!("Subcommand not defined: {}", name);
        }
    };

    ExitCode::from(return_code)
}

fn check_config_results(
    results: Vec<Result<(), RuntimeErr>>,
) -> Option<(u8, Vec<String>)> {
    let mut has_err = false;
    let mut messages = vec![];
    for result in results.iter() {
        if let Err(err) = result {
            has_err = true;
            messages.push(format!("{}", err));
        }
    }
    if has_err {
        Some((1, messages))
    } else {
        None
    }
}

/// Subcommand: run
fn handle_run(
    matches: &ArgMatches,
    max_call_depth: CallDepth,
    dis: bool,
    debug: bool,
) -> u8 {
    let file_name = matches.get_one::<String>("FILE_NAME");
    let code = matches.get_one::<String>("code");
    let history_path = matches.get_one::<String>("history_path");
    let save_repl_history = !matches.get_one::<bool>("no_history").unwrap();
    let argv = matches
        .get_many::<String>("argv")
        .unwrap_or_default()
        .map(|v| v.to_string())
        .collect();

    let result = if let Some(code) = code {
        if let Some(file_name) = file_name {
            let mut new_argv = vec![file_name.to_owned()];
            new_argv.extend(argv);
            run::run_text(code, max_call_depth, new_argv, dis, debug)
        } else {
            run::run_text(code, max_call_depth, argv, dis, debug)
        }
    } else if let Some(file_name) = file_name {
        if file_name == "-" {
            run::run_stdin(max_call_depth, argv, dis, debug)
        } else {
            let path = get_script_file_path(file_name);
            run::run_file(&path, max_call_depth, argv, dis, debug)
        }
    } else {
        match save_repl_history {
            true => {
                let history_path = create_repl_history_file(history_path);
                repl::run(history_path, argv, dis, debug)
            }
            false => repl::run(None, argv, dis, debug),
        }
    };

    match result {
        Ok(Some(message)) => {
            println!("{message}");
            0
        }
        Ok(None) => 0,
        Err((code, Some(message))) => {
            eprintln!("{message}");
            code
        }
        Err((code, None)) => code,
    }
}

/// Subcommand: test
fn handle_test(
    _matches: &ArgMatches,
    _max_call_depth: CallDepth,
    _dis: bool,
    _debug: bool,
) -> u8 {
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
fn create_repl_history_file(path: Option<&String>) -> Option<PathBuf> {
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
