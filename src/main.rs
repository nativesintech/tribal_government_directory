use std::error::Error;

mod lib;

//  Go to the page for the tribal directory: https://www.ncai.org/tribal-directory?page=1
//  Scape all the tribal information
//  <article class="clearfix"
//    <h2> {name} <span> {specifier} </span> </h2>
//    <p>  {contact} ... Recognition Status: {status: federal/state} </p>
//    <p class="right"> {address} ... Website: {website} </p>
//  </article>

//  <a class="next_page">Next --> </a>
//  <span class="next_page disabled">Next --></span>

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    lib::scrape_tribal_dir().await?;
    Ok(())
}
