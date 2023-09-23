//! tgd - Tribal Government Directory
//!
//! A command line utility (cli) to query a directory of tribal governments
//!
//! Examples:
//! ```console
//! $ tgd list # lists the names of all tribal governments
//! $ tgd list --name Muscogee # lists tribal governments with given name
//! $ tgd list --websites https # lists tribal governments who have https websites
//! $ tgd --help # more details about how it works
//! ```

use cli_table::{format::Justify, Cell, Style, Table};
use regex::Regex;
use select::document::Document;
use select::predicate::{Attr, Class, Name};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::path::Path;
use tokio::fs;

pub mod args;

// Scrape tribal information based on this HTML structure:
//   <article class="clearfix"
//     <h2> {name} <span> {specifier} </span> </h2>
//     <p>  {contact} ... Recognition Status: {status: federal/state} </p>
//     <p class="right"> {address} ... Website: {website} </p>
//   </article>
fn select_html(res: &str) -> Vec<Vec<String>> {
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
        let name_vec: Vec<&str> = name.split('[').collect();

        /* Get nation without region */
        let name_without_region: String = name_vec
            .first()
            .map(|v: &&str| v.trim_end())
            .unwrap_or("")
            .to_owned();
        data.push(name_without_region);

        /* Get region */
        let region: String = name_vec
            .first()
            .map(|v: &&str| v.trim_end_matches(|c| c == ' ' || c == ']' || c == '\n'))
            .unwrap_or("")
            .to_owned();
        data.push(region);

        /* Get recognition */
        let contact = node.find(Name("p")).next().unwrap().text();
        let recognition_vec: Vec<&str> = contact
            .split('\n')
            .filter(|v| v.contains("Recognition"))
            .map(|v| v.trim())
            .collect();
        let status: String = recognition_vec
            .first()
            .and_then(|v: &&str| v.get(20..))
            .unwrap_or("")
            .to_owned();
        data.push(status);

        /* Get website */
        let info = node.find(Attr("class", "right")).next().unwrap().text();
        let mut contact_vec: Vec<&str> = info.split('\n').map(|v| v.trim()).collect();
        let website = contact_vec.split_off(2);

        /* Get address */
        let address = contact_vec.join(", ");
        let address_regex = Regex::new(
            r"(?P<addr>[\w|\W]+),\s(?P<city>[\w|\W]+),\s(?P<state>[A-Z]{2})(?P<zip>\d+|\d+\-\d+)",
        )
        .unwrap();
        let next_addr = address_regex
            .replace_all(&address, "$addr $city, $state $zip")
            .to_string();
        data.push(next_addr);

        /* Get website */
        let site = website
            .join(" ")
            .get(8..)
            .map(|v| v.trim())
            .unwrap_or("")
            .to_owned();
        data.push(site);
    }

    let chunks = data.chunks(5).map(|c| c.into()).collect();

    chunks
}

#[derive(Debug)]
struct FileExistsError;

impl Error for FileExistsError {}

impl fmt::Display for FileExistsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The file ./tribes.csv already exits. If you want to overwrite this file use `tgd update --force`")
    }
}

/// Go to the page for the tribal directory (https://www.ncai.org/tribal-directory?page=1)
/// and output the data into a CSV file
pub async fn scrape_tribal_dir(force_flag: bool) -> Result<(), Box<dyn Error>> {
    // Check if tribes.csv exists, if it does and user uses --force flag, remove it and create a new file, otherwise exit saying that the file already exists
    let file_exists = Path::new("./tribes.csv").exists();

    if file_exists && !force_flag {
        return Err(Box::new(FileExistsError {}));
    }

    // Remove existing csv file
    fs::remove_file("./tribes.csv").await?;

    println!("💻 Requesting tribal directory from https://naci.org/tribal-directory");
    let mut wtr = csv::WriterBuilder::new()
        .flexible(true)
        .from_path("tribes.csv")?;

    /* Create columns for csv */
    wtr.write_record(["Nation", "Region", "Recognition", "Address", "Website"])?;

    println!("💿 Parsing HTML");
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
    println!("💾 Saved file to ./tribes.csv");
    Ok(())
}

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

/// Take the CSV data and output all the governments
pub fn list_govts() {
    let mut rdr = csv::Reader::from_path("./tribes.csv").expect("File tribes.csv does not exist");
    let mut table = Vec::new();
    for n in rdr.deserialize() {
        let nation: Nation = n.unwrap();
        let row = vec![nation.nation.cell().justify(Justify::Left)];
        table.push(row);
    }

    let t = table
        .table()
        .title(vec!["Name".cell().bold(true)])
        .display()
        .unwrap();

    println!("{}", t);
}

/// Filter the governments based on website filter, state, and name
pub fn filter_govts(
    filter: &Option<args::WebsiteFilter>,
    state: &Option<String>,
    name: &Option<String>,
) {
    let mut rdr = csv::Reader::from_path("./tribes.csv")
        .expect("File tribes.csv does not exist. Run `tgd update`");
    let mut data = Vec::new();

    for n in rdr.deserialize() {
        let nation: Nation = n.unwrap();
        data.push(nation);
    }

    if let Some(n) = name {
        data = data
            .into_iter()
            .filter(|nation| nation.nation.contains(n))
            .collect();
    }

    if let Some(s) = state {
        data = data.into_iter().filter(|n| n.address.contains(s)).collect();
    }

    if let Some(f) = filter {
        match f {
            args::WebsiteFilter::DotGov => {
                data = data
                    .into_iter()
                    .filter(|n| n.website.ends_with(".gov"))
                    .collect();
            }
            args::WebsiteFilter::DotCom => {
                data = data
                    .into_iter()
                    .filter(|n| n.website.ends_with(".com"))
                    .collect();
            }
            args::WebsiteFilter::DotOrg => {
                data = data
                    .into_iter()
                    .filter(|n| n.website.ends_with(".org"))
                    .collect();
            }
            args::WebsiteFilter::DotNet => {
                data = data
                    .into_iter()
                    .filter(|n| n.website.ends_with(".net"))
                    .collect();
            }
            args::WebsiteFilter::Http => {
                data = data
                    .into_iter()
                    .filter(|n| n.website.starts_with("http:"))
                    .collect();
            }
            args::WebsiteFilter::Https => {
                data = data
                    .into_iter()
                    .filter(|n| n.website.starts_with("https:"))
                    .collect();
            }
        }
    }

    let mut table = Vec::new();

    for nation in &data {
        let row = vec![
            nation.nation.as_str().cell().justify(Justify::Left),
            nation.website.as_str().cell().justify(Justify::Left),
        ];
        table.push(row);
    }

    let t = table
        .table()
        .title(vec!["Name".cell().bold(true), "Website".cell().bold(true)])
        .display()
        .unwrap();

    println!("{}", t);
}

/// Get basic stats about each tribal government
pub fn stats(filter: &Option<args::WebsiteFilter>) {
    let mut rdr = csv::Reader::from_path("./tribes.csv")
        .expect("File tribes.csv does not exist. Run ``tgd update`");
    let mut data = Vec::new();

    let mut number_of_nations = 0;
    let mut number_of_websites = 0;
    let mut result = "".to_owned();
    let mut percent_websites = "".to_owned();
    let mut percent_nations = "".to_owned();

    if let Some(f) = filter {
        for n in rdr.deserialize() {
            let nation: Nation = n.unwrap();
            if nation.recognition == "Federal" {
                number_of_nations += 1;

                if !nation.website.is_empty() {
                    number_of_websites += 1;
                }
            }
            data.push(nation);
        }

        match f {
            args::WebsiteFilter::DotGov => {
                data = data
                    .into_iter()
                    .filter(|n| n.website.ends_with(".gov"))
                    .collect();

                result.push_str("sites with dot gov domains: ");
                result.push_str(data.len().to_string().as_str());

                let pw = data.len() * 100 / number_of_websites;
                let pn = data.len() * 100 / number_of_nations;

                percent_websites.push_str(pw.to_string().as_str());
                percent_websites.push_str("%");
                percent_nations.push_str(pn.to_string().as_str());
                percent_nations.push_str("%");
            }
            args::WebsiteFilter::DotCom => {
                data = data
                    .into_iter()
                    .filter(|n| n.website.ends_with(".com"))
                    .collect();

                result.push_str("sites with dot com domains: ");
                result.push_str(data.len().to_string().as_str());

                let pw = data.len() * 100 / number_of_websites;
                let pn = data.len() * 100 / number_of_nations;

                percent_websites.push_str(pw.to_string().as_str());
                percent_nations.push_str(pn.to_string().as_str());
            }
            args::WebsiteFilter::DotOrg => {
                data = data
                    .into_iter()
                    .filter(|n| n.website.ends_with(".org"))
                    .collect();

                result.push_str("sites with dot org domains: ");
                result.push_str(data.len().to_string().as_str());

                let pw = data.len() * 100 / number_of_websites;
                let pn = data.len() * 100 / number_of_nations;

                percent_websites.push_str(pw.to_string().as_str());
                percent_websites.push_str("%");
                percent_nations.push_str(pn.to_string().as_str());
                percent_nations.push_str("%");
            }
            args::WebsiteFilter::DotNet => {
                data = data
                    .into_iter()
                    .filter(|n| n.website.ends_with(".net"))
                    .collect();

                result.push_str("sites with dot net domains: ");
                result.push_str(data.len().to_string().as_str());

                let pw = data.len() * 100 / number_of_websites;
                let pn = data.len() * 100 / number_of_nations;

                percent_websites.push_str(pw.to_string().as_str());
                percent_websites.push_str("%");
                percent_nations.push_str(pn.to_string().as_str());
                percent_nations.push_str("%");
            }
            args::WebsiteFilter::Http => {
                data = data
                    .into_iter()
                    .filter(|n| n.website.starts_with("http:"))
                    .collect();

                result.push_str("sites with http: ");
                result.push_str(data.len().to_string().as_str());

                let pw = data.len() * 100 / number_of_websites;
                let pn = data.len() * 100 / number_of_nations;

                percent_websites.push_str(pw.to_string().as_str());
                percent_websites.push_str("%");
                percent_nations.push_str(pn.to_string().as_str());
                percent_nations.push_str("%");
            }
            args::WebsiteFilter::Https => {
                data = data
                    .into_iter()
                    .filter(|n| n.website.starts_with("https:"))
                    .collect();

                result.push_str("sites with https: ");
                result.push_str(data.len().to_string().as_str());

                let pw = data.len() * 100 / number_of_websites;
                let pn = data.len() * 100 / number_of_nations;

                percent_websites.push_str(pw.to_string().as_str());
                percent_websites.push_str("%");
                percent_nations.push_str(pn.to_string().as_str());
                percent_nations.push_str("%");
            }
        }
    }

    println!("\n");
    println!("{result}");
    println!("percent of all websites: {percent_websites}");
    println!("percent of all nations: {percent_nations}");
}
