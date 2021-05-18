/*!
Interpreter

- Everything is an object of some type.
- Avoid keywords.
- Everything is immutable by default.
- Disallow arbitrary attachment of attributes.
- Lexical scoping.
- No this/self on methods but this/self is required to access
  attributes.

Builtin Types

- Bool
- Int
- Float
- String
- Char?
- None
- Some
- Option

Types

- Upper camel case only

    <Name> ([args])

        @new (value)
            this.value = value

    > Name.new(value)

Functions

- Lower snake case only

    <name> ([args]) [-> T]
        <body>

    <name> = ([args]) [-> T] <body>

    <name> = ([args]) [-> T]
        <body>

Loops

    i <- 0..10
        print(i)

*/
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
                .help("Script"),
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
        run::run(code, debug)
    } else if let Some(file_name) = file_name {
        run::run_file(file_name, debug)
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
