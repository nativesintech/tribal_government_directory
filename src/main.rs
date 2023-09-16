use clap::Parser;
use std::error::Error;
use tgd::{args, filter_govts, list_govts, scrape_tribal_dir, stats};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _output_file_path = "./tribes.csv";

    // tgd list will list all government names
    // tgd find <name> will do a simple search for a government based on the string provided and return all data about
    // that government, if one is not found a message will be returned that no government was found
    // tgd list --websites will list all websites not failing, since they are valid websites
    // tgd list --websites failing will list all failing sites
    // tgd update will call the command to scrape data from NCAI even if a file exists, will also update the
    // failed sites json
    // tgd list --websites dot-gov will list websites using a .gov domain and a table with statistics
    // tgd list --websites dot-us will list websites using the .us.gov domain and a table with statistics
    // tgd list --websites http will list websites using http and a table with statistics (percentages)
    // tgd list --websites https  will list websites using https and a table with statistics
    // tgd list --websites dot-gov,dot-us,http will list websites using this and a table of statistics with a column for each item in the list of items

    // scrape_tribal_dir().await?;
    // sites_with_nsngov();
    // check_websites().await?;

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
        args::Commands::Update {
            latest: _,
            force: _,
        } => {
            scrape_tribal_dir().await?;
        }
    }

    Ok(())
}
