use html_parser::Dom;

#[test]
fn raw_attribute() {
    let html = r#"
    <div>
        <div unrecognizedattribute="asd">hello world!</div>
    </div>
    "#
    .trim();

    let dom = Dom::parse(html).unwrap();

    let body = aether_rsx_rosetta::rsx_from_html(&dom);

    let out = aether_autofmt::write_block_out(&body).unwrap();

    let expected = r#"
    div {
        div { "unrecognizedattribute": "asd", "hello world!" }
    }"#;
    pretty_assertions::assert_eq!(&out, &expected);
}
