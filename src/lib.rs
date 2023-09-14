use cli_table::{format::Justify, print_stdout, Cell, Style, Table};
use regex::Regex;
use reqwest::StatusCode;
use select::document::Document;
use select::predicate::{Attr, Class, Name};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::ops::Deref;

pub mod args;

/// Scrape tribal information based on this HTML structure:
///   <article class="clearfix"
///     <h2> {name} <span> {specifier} </span> </h2>
///     <p>  {contact} ... Recognition Status: {status: federal/state} </p>
///     <p class="right"> {address} ... Website: {website} </p>
///   </article>
pub fn select_html(res: &str) -> Vec<Vec<String>> {
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

/// Go to the page for the tribal directory (https://www.ncai.org/tribal-directory?page=1)
/// and output the data into a CSV
pub async fn scrape_tribal_dir() -> Result<(), Box<dyn Error>> {
    /* Create writer */
    let mut wtr = csv::WriterBuilder::new()
        .flexible(true)
        .from_path("tribes.csv")?;

    /* Create columns for csv */
    wtr.write_record(["Nation", "Region", "Recognition", "Address", "Website"])?;

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

pub fn filter_govts(filter: &args::WebsiteFilter) {
    let mut rdr = csv::Reader::from_path("./tribes.csv").expect("File tribes.csv does not exist");
    let mut data = Vec::new();

    for n in rdr.deserialize() {
        let nation: Nation = n.unwrap();
        data.push(nation);
    }

    match filter {
        args::WebsiteFilter::DotGov => {
            data = data
                .into_iter()
                .filter(|d| d.website.ends_with(".gov"))
                .collect();
        }
        args::WebsiteFilter::DotCom => {
            data = data
                .into_iter()
                .filter(|d| d.website.ends_with(".com"))
                .collect();
        }
        args::WebsiteFilter::DotOrg => {
            data = data
                .into_iter()
                .filter(|d| d.website.ends_with(".org"))
                .collect();
        }
        args::WebsiteFilter::DotNet => {
            data = data
                .into_iter()
                .filter(|d| d.website.ends_with(".net"))
                .collect();
        }
        args::WebsiteFilter::Http => {
            data = data
                .into_iter()
                .filter(|d| d.website.starts_with("http:"))
                .collect();
        }
        args::WebsiteFilter::Https => {
            data = data
                .into_iter()
                .filter(|d| d.website.starts_with("https:"))
                .collect();
        }
        args::WebsiteFilter::Failing => {}
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

/// Take the JSON and do some simple analytics
pub fn sites_with_nsngov() {
    let str = fs::read_to_string("tribes.json").expect("Unable to read file");
    let data: Vec<Nation> = serde_json::from_str(&str).unwrap();

    let mut number_of_nations = 0;
    let mut number_of_websites = 0;
    let mut number_of_websites_with_nsn = 0;
    let mut number_of_https = 0;

    for nation in data.iter() {
        if nation.recognition == "Federal" {
            number_of_nations += 1;

            if !nation.website.is_empty() {
                number_of_websites += 1;

                if nation.website.starts_with("https") {
                    number_of_https += 1;
                }

                if nation.website.contains(".gov") {
                    number_of_websites_with_nsn += 1;
                }
            }
        }
    }

    println!("nation: {number_of_nations}");
    println!("with sites: {number_of_websites}",);
    println!("with nsn-gov: {number_of_websites_with_nsn}",);
    println!("with https: {number_of_https}",)
}

/// Go through the websites and see which ones work and which ones redirect
pub async fn check_websites() -> Result<(), Box<dyn Error>> {
    let str = fs::read_to_string("tribes.json").expect("Unable to read file");
    let data: Vec<Nation> = serde_json::from_str(&str).unwrap();

    for nation in data.iter() {
        let site = &nation.website;

        if site.is_empty() {
            continue;
        }

        if nation.recognition != "Federal" {
            continue;
        }

        let res = reqwest::get(site).await;

        match res {
            Ok(resp) => {
                let status = resp.status();
                let headers = resp.headers();

                match status {
                    StatusCode::MOVED_PERMANENTLY => {
                        let location = headers.get("Location").unwrap().to_str().unwrap();
                        println!("site: {site}, status: Moved Permanently, location: {location}");
                    }
                    StatusCode::TEMPORARY_REDIRECT => {
                        let location = headers.get("Location").unwrap().to_str().unwrap();
                        println!("site: {site}, status: Temporary Redirect, location: {location}");
                    }
                    StatusCode::PERMANENT_REDIRECT => {
                        let location = headers.get("Location").unwrap().to_str().unwrap();
                        println!("site: {site}, status: Permanent Redirect, location: {location}");
                    }
                    StatusCode::BAD_GATEWAY => {
                        println!("site: {site}, status: Bad Gateway");
                    }
                    StatusCode::BAD_REQUEST => {
                        println!("site: {site}, status: Bad Request");
                    }
                    StatusCode::GATEWAY_TIMEOUT => {
                        println!("site: {site}, status: Gateway Timeout");
                    }
                    StatusCode::INTERNAL_SERVER_ERROR => {
                        println!("site: {site}, status: Internal Server Error");
                    }
                    StatusCode::SERVICE_UNAVAILABLE => {
                        println!("site: {site}, status: Service Unavailable");
                    }
                    StatusCode::OK => continue,
                    _ => {
                        println!("site: {site}, status: {status}");
                    }
                }
            }
            Err(err) => {
                let str_err = err.to_string();
                println!("site: {site}, error: {str_err}");
                continue;
            }
        }
    }

    Ok(())
}
