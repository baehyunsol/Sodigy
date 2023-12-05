use crate::try_intern_short_string;

#[test]
fn short_strings() {
    let samples = vec![
        "🦫", "가", "", " ", "  ",
        "a", "b", "abc", "...", "_",
    ];

    for sample in samples.into_iter() {
        let i = try_intern_short_string(sample.as_bytes()).unwrap();
        let (len, s) = i.try_unwrap_short_string().unwrap();

        assert_eq!(sample.as_bytes(), &s[0..len]);
    }
}
