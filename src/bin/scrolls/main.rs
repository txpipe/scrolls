use clap::Command;
use std::process;

mod daemon;

fn main() {
    let args = Command::new("app")
        .name("scrolls")
        .about("cardano cache")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(daemon::command_definition())
        .arg_required_else_help(true)
        .get_matches();

    let result = match args.subcommand() {
        Some(("daemon", args)) => daemon::run(args),
        _ => Err(scrolls::Error::ConfigError("nothing to do".to_string())),
    };

    if let Err(err) = &result {
        eprintln!("ERROR: {:#?}", err);
        process::exit(1);
    }

    process::exit(0);
}
