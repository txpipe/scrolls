use clap::Parser;
use miette::Result;

mod scaffold;

#[derive(Parser)]
#[clap(name = "Scrolls")]
#[clap(bin_name = "scrolls-sdk")]
#[clap(author, version, about, long_about = None)]
enum ScrollsSdk {
    Scaffold(scaffold::Args),
}

fn main() -> Result<()> {
    let args = ScrollsSdk::parse();

    match args {
        ScrollsSdk::Scaffold(x) => scaffold::run(&x),
    }
}
