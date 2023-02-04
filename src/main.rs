use scrape_tribes_directory::{scrape_tribal_dir, sites_with_nsngov};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    scrape_tribal_dir().await?;
    sites_with_nsngov();
    Ok(())
}
