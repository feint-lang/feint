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
use std::process::exit;

use clap::{App, Arg};

use feint::run::Runner;

fn main() {
    let app = App::new("Interpreter")
        .version("0.0")
        .arg(
            Arg::with_name("FILE_NAME")
                .index(1)
                .required(false)
                .help("Script"),
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
    let debug = matches.is_present("debug");
    let mut interpreter = Runner::new(debug);

    let result = match file_name {
        Some(file_name) => interpreter.run(file_name),
        None => interpreter.repl(),
    };

    match result {
        Ok(Some(message)) => {
            println!("{}", message);
            exit(0);
        }
        Ok(None) => exit(0),
        Err((code, message)) => {
            eprintln!("{}", message);
            exit(code)
        }
    }
}
