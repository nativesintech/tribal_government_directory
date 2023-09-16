use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "Tribal Government Directory")]
#[command(version = "1.0.0")]
#[command(about = "A Directory of Tribal Governments")]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List tribal governments
    List {
        /// Filter by website protocol
        #[arg(short, long, value_enum, id = "FILTER")]
        websites: Option<WebsiteFilter>,

        /// Filter by state
        #[arg(short, long, id = "STATE")]
        state: Option<String>,

        /// Filter by name
        #[arg(short, long, id = "NAME")]
        name: Option<String>,
    },

    /// Update or add tribes.csv file
    Update {
        // Hidden command
        #[arg(hide = true, default_value = Some("true"))]
        latest: Option<bool>,

        /// Force update of tribes.csv even when it already exists
        #[arg(short, long, default_value = Some("true"))]
        force: Option<bool>,
    },

    /// Get stats for tribal governments
    Stats {
        /// Filter stats based on website filters
        #[arg(short, long)]
        filter: Option<WebsiteFilter>,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, ValueEnum)]
pub enum WebsiteFilter {
    /// Filter for .gov websites
    DotGov,
    /// Filter for .com websites
    DotCom,
    /// Filter for .net websites
    DotNet,
    /// Filter for .org websites
    DotOrg,
    /// Filter for http protocol websites
    Http,
    /// Filter for https protocol websites
    Https,
    //// Filter for unreachable/failing websites
    // Failing,
}
