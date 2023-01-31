use std::error::Error;

mod lib;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // lib::scrape_tribal_dir().await?;
    // lib::sites_with_nsngov();
    Ok(())
}
