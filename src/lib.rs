use regex::Regex;
use select::document::Document;
use select::predicate::{Attr, Class, Name};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
struct Nation {
    #[serde(alias = "Nation")]
    nation: String,
    #[serde(alias = "Region")]
    region: String,
    #[serde(alias = "Recognition")]
    recognition: String,
    #[serde(alias = "Address")]
    address: String,
    #[serde(alias = "Website")]
    website: String,
}

pub fn sites_with_nsngov() {
    let str = fs::read_to_string("tribes.json").expect("Unable to read file");
    let data: Vec<Nation> = serde_json::from_str(&str).unwrap();

    let mut number_of_nations = 0;
    let mut number_of_websites = 0;
    let mut number_of_websites_with_nsn = 0;

    for nation in data.iter() {
        if nation.recognition == "Federal" {
            number_of_nations += 1;

            if !nation.website.is_empty() {
                number_of_websites += 1;

                if nation.website.contains("-nsn.gov") {
                    number_of_websites_with_nsn += 1;
                }
            }
        }
    }

    println!("nation: {}", number_of_nations);
    println!("with sites: {}", number_of_websites);
    println!("with nsn-gov: {}", number_of_websites_with_nsn);
}

pub fn select_html(res: &str) -> Vec<Vec<String>> {
    /* Scrape tribal information based on this HTML structure:
        <article class="clearfix"
          <h2> {name} <span> {specifier} </span> </h2>
          <p>  {contact} ... Recognition Status: {status: federal/state} </p>
          <p class="right"> {address} ... Website: {website} </p>
        </article>
    */
    let document = Document::from(res);
    let articles = document
        .find(Name("article"))
        .into_selection()
        .filter(Class("clearfix"));

    let mut data: Vec<String> = vec![];

    /* Iterate on article tags to pluck out the gov information */
    for node in articles.into_iter() {
        /* Get nation with region */
        let name = node.find(Name("h2")).next().unwrap().text();
        let name_vec: Vec<&str> = name.split("[").collect();

        /* Get nation without region */
        let name_without_region: &str = name_vec.get(0).map(|v| v.as_ref()).unwrap();
        data.push(name_without_region.trim_end().to_owned());

        /* Get region */
        let region: &str = name_vec.get(1).map(|v| v.as_ref()).unwrap();
        data.push(
            region
                .trim_end_matches(|c| c == ' ' || c == ']' || c == '\n')
                .to_owned(),
        );

        /* Get recognition */
        let recog_regex = Regex::new(r"Recognition Status: (\w+)").unwrap();
        let contact = node.find(Name("p")).next().unwrap().text();
        for status in recog_regex.captures_iter(&contact) {
            let res = status.get(1).map_or("", |m| m.as_str());
            data.push(res.to_owned());
        }

        /* Get website */
        let info = node.find(Attr("class", "right")).next().unwrap().text();
        let mut contact_vec: Vec<&str> = info.split('\n').map(|v| v.trim()).collect();
        let website = contact_vec.split_off(2);

        let address = contact_vec.join(", ");

        println!("{:?}", address);

        /* Get address */
        data.push(address.to_owned());

        /* Get website */
        unsafe {
            let site = website.join(" ");
            let slice = site.get_unchecked(8..);
            data.push(slice.trim().to_owned());
        }
    }

    let chunks = data.chunks(5).map(|c| c.into()).collect();

    chunks
}

/*
  Go to the page for the tribal directory: https://www.ncai.org/tribal-directory?page=1
  and output the data into a CSV
*/
pub async fn scrape_tribal_dir() -> Result<(), Box<dyn Error>> {
    /* Create writer */
    let mut wtr = csv::WriterBuilder::new()
        .flexible(true)
        .from_path("tribes.csv")?;

    /* Create columns for csv */
    wtr.write_record(&["Nation", "Region", "Recognition", "Address", "Website"])?;

    for number in 1..=26 {
        /* Fetch html from ncai tribal directory from pages 1 - 26 (# of letters in alphabet)  */
        let res = reqwest::get(
            "https://www.ncai.org/tribal-directory?page=".to_owned() + &number.to_string(),
        )
        .await?
        .text()
        .await?;

        /* Get data for each csv column */
        let data = select_html(&res);

        /* Write data to tribes.csv */
        for d in data.iter() {
            wtr.write_record(d)?
        }
    }

    wtr.flush()?;
    Ok(())
}
