use std::process;

use clap::{App, Arg};

use feint::repl;
use feint::run;

/// Interpret a file if one is specified. Otherwise, run the REPL.
fn main() {
    let app = App::new("FeInt")
        .version("0.0.0")
        .arg(
            Arg::with_name("FILE_NAME")
                .index(1)
                .required(false)
                .conflicts_with("code")
                .help("Script file to run (use - to read from stdin)"),
        )
        .arg(
            Arg::with_name("code")
                .short("c")
                .long("code")
                .required(false)
                .conflicts_with("FILE_NAME")
                .takes_value(true)
                .help("Code to run"),
        )
        .arg(
            Arg::with_name("debug")
                .short("d")
                .long("debug")
                .required(false)
                .takes_value(false)
                .help("Enable debug mode?"),
        );

    let matches = app.get_matches();
    let file_name = matches.value_of("FILE_NAME");
    let code = matches.value_of("code");
    let debug = matches.is_present("debug");

    let result = if let Some(code) = code {
        run::run_text(code, debug)
    } else if let Some(file_name) = file_name {
        if file_name == "-" {
            run::run_stdin(debug)
        } else {
            run::run_file(file_name, debug)
        }
    } else {
        repl::run(debug)
    };

    match result {
        Ok(Some(message)) => exit(Some(message)),
        Ok(None) => exit(None),
        Err((code, message)) => error_exit(code, message),
    }
}

/// Exit 0 with optional message.
fn exit(message: Option<String>) {
    if message.is_some() {
        println!("{}", message.unwrap());
    }
    process::exit(0);
}

/// Exit with non-zero and error message.
fn error_exit(code: i32, message: String) {
    eprintln!("{}", message);
    process::exit(code);
}
