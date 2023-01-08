use std::path::PathBuf;
use std::process::ExitCode;

use clap::{parser::ValueSource, value_parser, Arg, ArgAction, ArgMatches, Command};

use feint::repl;
use feint::run;
use feint::vm::{CallDepth, DEFAULT_MAX_CALL_DEPTH};

/// Interpret a file if one is specified. Otherwise, run the REPL.
fn main() -> ExitCode {
    env_logger::init();

    // Args for run subcommand
    let no_history_arg = Arg::new("no_history")
        .long("no-history")
        .action(ArgAction::SetTrue)
        .help("Disable REPL history?");

    let file_name_arg = Arg::new("FILE_NAME")
        .index(1)
        .required(false)
        .conflicts_with("code")
        .help("Script file to run (use - to read from stdin)");

    let code_arg = Arg::new("code")
        .short('c')
        .long("code")
        .required(false)
        .conflicts_with("FILE_NAME")
        .num_args(1)
        .help("Code to run");

    let history_path_arg = Arg::new("history_path")
        .long("history-path")
        .required(false)
        .num_args(1)
        .help("Disable REPL history?");

    let argv_arg = Arg::new("argv").index(2).trailing_var_arg(true).num_args(0..);

    let app = Command::new("FeInt")
        .version("0.0.0")
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
        // Subcommand: run (when no subcommand is specified)
        .arg(&file_name_arg)
        .arg(&code_arg)
        .arg(&history_path_arg)
        .arg(&no_history_arg)
        .arg(&argv_arg)
        .subcommands([
            // Subcommand: run
            Command::new("run")
                .about("Run script or code")
                .arg(&file_name_arg)
                .arg(&code_arg)
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
    let max_call_depth = *matches.get_one("max_call_depth").unwrap();
    let dis = *matches.get_one::<bool>("dis").unwrap();
    let debug = *matches.get_one::<bool>("debug").unwrap();

    let max_call_depth = match matches.value_source("max_call_depth") {
        Some(ValueSource::DefaultValue) => DEFAULT_MAX_CALL_DEPTH,
        _ => max_call_depth,
    };

    let return_code = match matches.subcommand() {
        Some(("run", matches)) => handle_run(matches, max_call_depth, dis, debug),
        Some(("test", matches)) => handle_test(matches, max_call_depth, dis, debug),
        None => handle_run(&matches, max_call_depth, dis, debug),
        _ => {
            unreachable!("xxx");
        }
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

    let result = if let Some(file_name) = file_name {
        if file_name == "-" {
            run::run_stdin(max_call_depth, argv, dis, debug)
        } else {
            run::run_file(file_name, max_call_depth, argv, dis, debug)
        }
    } else if let Some(code) = code {
        run::run_text(code, max_call_depth, argv, dis, debug)
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
