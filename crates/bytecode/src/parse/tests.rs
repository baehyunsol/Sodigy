use super::parse;

const SAMPLE: &'static str = r#""#;

#[test]
fn parse_sample() {
    parse(SAMPLE.as_bytes()).unwrap();
}
