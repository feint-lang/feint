use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Arg, Command};

use feint::repl;
use feint::run;

/// Interpret a file if one is specified. Otherwise, run the REPL.
fn main() -> ExitCode {
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
                .takes_value(true)
                .help("Code to run"),
        )
        .arg(
            Arg::new("history_path")
                .long("history-path")
                .required(false)
                .takes_value(true)
                .help("Disable REPL history?"),
        )
        .arg(
            Arg::new("no_history")
                .long("no-history")
                .required(false)
                .takes_value(false)
                .help("Disable REPL history?"),
        )
        .arg(
            Arg::new("dis")
                .short('i')
                .long("dis")
                .required(false)
                .takes_value(false)
                .help("Disassemble instructions?"),
        )
        .arg(
            Arg::new("debug")
                .short('d')
                .long("debug")
                .required(false)
                .takes_value(false)
                .help("Enable debug mode?"),
        );

    let matches = app.get_matches();
    let file_name = matches.value_of("FILE_NAME");
    let code = matches.value_of("code");
    let history_path = matches.value_of("history_path");
    let save_repl_history = !matches.is_present("no_history");
    let dis = matches.is_present("dis");
    let debug = matches.is_present("debug");

    let result = if let Some(code) = code {
        run::run_text(code, dis, debug)
    } else if let Some(file_name) = file_name {
        if file_name == "-" {
            run::run_stdin(dis, debug)
        } else {
            run::run_file(file_name, dis, debug)
        }
    } else {
        match save_repl_history {
            true => {
                let history_path = match history_path {
                    Some(path) => PathBuf::from(path),
                    None => default_history_path(),
                };
                repl::run(Some(history_path.as_path()), dis, debug)
            }
            false => repl::run(None, dis, debug),
        }
    };

    let return_code = match result {
        Ok(Some(message)) => {
            println!("{}", message);
            0
        }
        Ok(None) => 0,
        Err((code, Some(message))) => {
            eprintln!("{}", message);
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
    let history_path_buf = base_path.join(".feint_history");
    history_path_buf
}
