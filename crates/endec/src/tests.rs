use crate::Endec;

#[test]
fn string_endec() {
    for s in [
        "",
        "a", "가",
        "abc", "가나다",
        "a가b나c다",
    ] {
        let s = s.to_string();
        let se = s.encode();
        let ss = String::decode(&se).unwrap();
        let sse = ss.encode();
        let sss = String::decode(&sse).unwrap();

        assert_eq!(s, ss);
        assert_eq!(se, sse);
        assert_eq!(ss, sss);
    }
}
