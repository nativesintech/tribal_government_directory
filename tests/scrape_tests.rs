use scrape_tribes_directory::select_html;

#[test]
fn scrape_test() {
    let actual: Vec<String> = select_html(include_str!("test.html"))
        .get(0)
        .unwrap()
        .to_vec();
    let expected: Vec<String> = vec![
        "Absentee-Shawnee Tribe of Indians of Oklahoma", // Nation
        "Southern Plains",                               // Region
        "Federal",                                       // Recognition
        "2025 S. Gordon Cooper Drive Shawnee, OK 74801-9005", // Address
        "http://www.astribe.com",                        // Website (if one)
    ]
    .into_iter()
    .map(|v| v.to_string())
    .collect();
    assert_eq!(actual, expected)
}

#[test]
fn sites_with_nsngov_test() {}
