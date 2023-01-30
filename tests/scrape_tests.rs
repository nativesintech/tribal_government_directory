use scrape_tribes_directory::select_html;

#[test]
fn scrape_test() {
    let actual: Vec<String> = select_html(include_str!("test.html"))
        .into_iter()
        .take(4)
        .collect();
    let expected: Vec<String> = vec![
        "Absentee-Shawnee Tribe of Indians of Oklahoma",
        "Southern Plains",
        "Federal",
        "http://www.astribe.com",
    ]
    .into_iter()
    .map(|v| v.to_string())
    .collect();
    assert_eq!(actual, expected)
}
