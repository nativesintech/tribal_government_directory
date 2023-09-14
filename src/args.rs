use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "Tribal Government Directory")]
#[command(version = "1.0")]
#[command(about = "A Directory of Tribal Governments that you can filter")]
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
    /// Filter for unreachable/failing websites
    Failing,
}
