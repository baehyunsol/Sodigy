pub fn fold_if_long(html: String, len: usize) -> String {
    if len > 5 {
        todo!()
    } else {
        html
    }
}

pub fn escape_html(s: &str) -> String {
    s
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
}

