use clap::builder::FalseyValueParser;
use clap::{value_parser, Arg, ArgAction, Command};

pub fn build_cli() -> Command {
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

    Command::new("FeInt")
        .version("0.0.0")
        .arg(
            Arg::new("max_call_depth")
                .short('x')
                .long("max-call-depth")
                .default_value("0")
                .value_parser(value_parser!(usize))
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
            Command::new("test")
                .about("Run test")
                .arg(Arg::new("argv").index(1).trailing_var_arg(true).num_args(0..)),
        ])
}
