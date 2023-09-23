use clap::Parser;
use std::error::Error;
use tgd::{args, filter_govts, list_govts, scrape_tribal_dir, stats};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _output_file_path = "./tribes.csv";

    let cli = args::Cli::parse();

    match &cli.command {
        args::Commands::List {
            websites,
            state,
            name,
        } => match (&websites, &state, &name) {
            (None, None, None) => {
                list_govts();
            }
            (filter, state, name) => {
                filter_govts(filter, state, name);
            }
        },
        args::Commands::Stats { filter } => match &filter {
            _ => stats(filter),
        },
        args::Commands::Update { latest: _, force } => {
            scrape_tribal_dir(*force.clone().get_or_insert(false)).await?;
        }
    }

    Ok(())
}
