use std::path::PathBuf;
use std::process::ExitCode;

use clap::{parser::ValueSource, value_parser, Arg, ArgAction, Command};

use feint::repl;
use feint::run;
use feint::vm::{CallDepth, DEFAULT_MAX_CALL_DEPTH};

/// Interpret a file if one is specified. Otherwise, run the REPL.
fn main() -> ExitCode {
    env_logger::init();

    let app = Command::new("FeInt")
        .version("0.0.0")
        .arg(
            Arg::new("FILE_NAME")
                .index(1)
                .required(false)
                .conflicts_with("code")
                .help("Script file to run (use - to read from stdin)"),
        )
        .arg(
            Arg::new("code")
                .short('c')
                .long("code")
                .required(false)
                .conflicts_with("FILE_NAME")
                .num_args(1)
                .help("Code to run"),
        )
        .arg(
            Arg::new("history_path")
                .long("history-path")
                .required(false)
                .num_args(1)
                .help("Disable REPL history?"),
        )
        .arg(
            Arg::new("no_history")
                .long("no-history")
                .action(ArgAction::SetTrue)
                .help("Disable REPL history?"),
        )
        .arg(
            Arg::new("max_call_depth")
                .short('m')
                .long("max-call-depth")
                .default_value("0")
                .value_parser(value_parser!(CallDepth))
                .help("Maximum call/recursion depth"),
        )
        .arg(
            Arg::new("dis")
                .short('i')
                .long("dis")
                .action(ArgAction::SetTrue)
                .help("disassemble instructions?"),
        )
        .arg(
            Arg::new("debug")
                .short('d')
                .long("debug")
                .action(ArgAction::SetTrue)
                .help("Enable debug mode?"),
        )
        .arg(Arg::new("argv").index(2).trailing_var_arg(true).num_args(0..));

    let matches = app.get_matches();
    let file_name = matches.get_one::<String>("FILE_NAME");
    let code = matches.get_one::<String>("code");
    let history_path = matches.get_one::<String>("history_path");
    let save_repl_history = !matches.get_one::<bool>("no_history").unwrap();
    let max_call_depth = *matches.get_one("max_call_depth").unwrap();
    let dis = *matches.get_one::<bool>("dis").unwrap();
    let debug = *matches.get_one::<bool>("debug").unwrap();
    let argv = matches
        .get_many::<String>("argv")
        .unwrap_or_default()
        .map(|v| v.to_string())
        .collect();

    let max_call_depth = match matches.value_source("max_call_depth") {
        Some(ValueSource::DefaultValue) => DEFAULT_MAX_CALL_DEPTH,
        _ => max_call_depth,
    };

    let result = if let Some(code) = code {
        run::run_text(code, max_call_depth, argv, dis, debug)
    } else if let Some(file_name) = file_name {
        if file_name == "-" {
            run::run_stdin(max_call_depth, argv, dis, debug)
        } else {
            run::run_file(file_name, max_call_depth, argv, dis, debug)
        }
    } else {
        match save_repl_history {
            true => {
                let history_path =
                    history_path.map_or_else(default_history_path, PathBuf::from);
                repl::run(Some(history_path), argv, dis, debug)
            }
            false => repl::run(None, argv, dis, debug),
        }
    };

    let return_code = match result {
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
    };

    ExitCode::from(return_code)
}

/// Get the default history path, which is either ~/.feint_history or,
/// if the user's home directory can't be located, ./.feint_history.
fn default_history_path() -> PathBuf {
    let home = dirs::home_dir();
    let base_path = home.unwrap_or_default();
    base_path.join(".feint_history")
}
