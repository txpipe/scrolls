use clap::Parser;
use std::process;

mod console;
mod daemon;

#[derive(Parser)]
#[clap(name = "Scrolls")]
#[clap(bin_name = "scrolls")]
#[clap(author, version, about, long_about = None)]
enum Scrolls {
    Daemon(daemon::Args),
}

fn main() {
    let args = Scrolls::parse();

    let result = match args {
        Scrolls::Daemon(x) => daemon::run(&x),
    };

    if let Err(err) = &result {
        eprintln!("ERROR: {:#?}", err);
        process::exit(1);
    }

    process::exit(0);
}
