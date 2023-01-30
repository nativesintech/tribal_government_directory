use regex::Regex;
use select::document::Document;
use select::predicate::{Attr, Class, Name};
use std::error::Error;

pub fn select_html(res: &str) -> Vec<String> {
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
        /* Get name with region */
        let name = node.find(Name("h2")).next().unwrap().text();
        let name_vec: Vec<&str> = name.split("[").collect();

        /* Get name without region */
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
            let res = status.get(1).map_or("No Status", |m| m.as_str());
            data.push(res.to_owned());
        }

        /* Get website */
        let web_regex = Regex::new(r"Website: ([\s\S]*)").unwrap();
        let info = node.find(Attr("class", "right")).next().unwrap().text();
        for site in web_regex.captures_iter(&info) {
            let copy = site.get(1).map_or("No Website", |m| m.as_str());
            data.push(copy.to_owned());
        }
    }

    data
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
    wtr.write_record(&["Nation", "Region", "Recognition", "Website"])?;

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
        wtr.write_record(&data)?;
    }

    wtr.flush()?;
    Ok(())
}
