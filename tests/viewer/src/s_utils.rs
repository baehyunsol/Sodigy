pub fn trim_if_too_long(s: &str, len: usize) -> String {
    if s.chars().count() > len + 3 {
        format!("{}...", s.chars().take(len).collect::<String>())
    } else {
        s.to_string()
    }
}

pub fn render_ms(ms: u64) -> String {
    if ms < 10_000 {
        format!("{}.{:03} seconds", ms / 1000, ms % 1000)
    } else if ms < 120_000 {
        format!("{} seconds", ms / 1000)
    } else {
        format!("{} minutes {} seconds", ms / 60_000, ms / 1000 % 60)
    }
}

