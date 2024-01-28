use std::{fs::File, path::PathBuf};

use handlebars::Handlebars;
use miette::{IntoDiagnostic, Result};
use serde_json::json;
use walkdir::WalkDir;

#[derive(clap::Args)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap()]
    name: String,

    #[clap(long, short)]
    template: PathBuf,

    #[clap(long, short)]
    output: Option<PathBuf>,

    #[clap(long, short)]
    db_connection: Option<String>,

    #[clap(long, short)]
    u5c_endpoint: Option<String>,

    #[clap(long, env = "SCROLLS_SDK_TEMPLATE_ROOT")]
    template_root: Option<PathBuf>,
}

pub fn run(args: &Args) -> Result<()> {
    let mut handlebars = Handlebars::new();

    let data = json!({
        "reducer": args.name,
        "db_connection": args.u5c_endpoint.clone().unwrap_or("sqlite://./scrolls.db".into()),
        "u5c_endpoint": args.u5c_endpoint.clone().unwrap_or("mainnet.utxorpc-v0.demeter.run".into()),
    });

    let template_path = args
        .template_root
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join(args.template.clone());

    let source = WalkDir::new(&template_path);

    let output = args.output.clone().unwrap_or(
        std::env::current_dir()
            .into_diagnostic()?
            .join(PathBuf::from(&args.name)),
    );

    std::fs::create_dir_all(&output).into_diagnostic()?;

    for entry in source {
        let entry = entry.into_diagnostic()?;

        let relative = entry
            .path()
            .strip_prefix(&template_path)
            .into_diagnostic()?;

        if entry.path().is_dir() {
            let output = output.join(relative);
            std::fs::create_dir_all(output).into_diagnostic()?;
        } else if entry.path().extension().is_some_and(|p| p == "hbs") {
            let output = output.join(relative.file_stem().unwrap());
            handlebars
                .register_template_file("template", entry.path())
                .into_diagnostic()?;

            let mut output = File::create(&output).into_diagnostic()?;
            handlebars
                .render_to_write("template", &data, &mut output)
                .into_diagnostic()?;
        } else {
            let output = output.join(relative);
            std::fs::copy(entry.path(), output).into_diagnostic()?;
        }
    }

    println!("scaffold complete");

    Ok(())
}
