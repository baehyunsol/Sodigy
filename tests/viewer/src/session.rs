use crate::{Color, Id, escape_html};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize)]
pub struct Session {
    // These data have to be collected before any nodes are rendered.
    pub title: HashMap<Id, (String, Option<Color>)>,
    pub children: HashMap<Id, Option<Vec<(String, Id)>>>,
    pub parent: HashMap<Id, Vec<(Id, usize)>>,
    pub comparisons: HashMap<Id, Vec<(String, Id)>>,

    pub leaves: HashMap<Id, Vec<(Vec<String>, Option<Color>, Id)>>,
    pub config: Config,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Config {
    pub fold_threshold: usize,
    pub main_title_length_limit: Option<usize>,
    pub entry_title_length_limit: Option<usize>,
}

impl Session {
    pub fn new(config: Config) -> Session {
        Session {
            title: HashMap::new(),
            children: HashMap::new(),
            parent: HashMap::new(),
            comparisons: HashMap::new(),
            leaves: HashMap::new(),
            config,
        }
    }

    pub fn fold_if_long(&self, html: String, len: usize) -> String {
        if len > self.config.fold_threshold {
            format!("<details><summary>{len} elements</summary>{html}</details>")
        } else {
            html
        }
    }

    pub fn render_entry(&self, title: &str, id: Id) -> String {
        let (_, flag) = self.title.get(&id).unwrap();
        let flag = if let Some(flag) = flag {
            format!(r#"<span class="{}"></span>"#, flag.small_flag())
        } else {
            String::new()
        };
        format!(r#"<a href="{id}.html">{}</a>{flag}"#, self.render_title(title, self.config.entry_title_length_limit))
    }

    pub fn render_title(&self, title: &str, limit: Option<usize>) -> String {
        match (limit, title.contains("[([[") && title.contains("]])]")) {
            (limit, true) => apply_markup(title, limit),
            (Some(limit), false) if title.chars().count() > limit => escape_html(&format!("{}...", title.chars().take(limit.max(3) - 3).collect::<String>())),
            _ => escape_html(title),
        }
    }

    pub fn check_filter(&self, filter: Option<Color>, id: Id) -> bool {
        match (filter, self.title.get(&id)) {
            (None, _) => true,
            (Some(_), Some((_, None))) => false,
            (Some(filter), Some((_, Some(flag)))) => filter == *flag,
            _ => unreachable!(),
        }
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            fold_threshold: 5,
            main_title_length_limit: Some(88),
            entry_title_length_limit: Some(48),
        }
    }
}

enum Token {
    Content(String),
    Tag(String),
}

fn apply_markup(title: &str, limit: Option<usize>) -> String {
    let bytes = title.as_bytes();
    let mut cursor = 0;
    let mut buffer = vec![];
    let mut tokens = vec![];

    loop {
        match (
            bytes.get(cursor),
            bytes.get(cursor + 1),
            bytes.get(cursor + 2),
            bytes.get(cursor + 3),
        ) {
            (Some(b'['), Some(b'('), Some(b'['), Some(b'[')) => {
                cursor += 4;

                match parse_tag_name(bytes, cursor) {
                    Some((tag, new_cursor)) => {
                        cursor = new_cursor;
                        tokens.push(Token::Content(String::from_utf8_lossy(&buffer).to_string()));
                        tokens.push(Token::Tag(String::from_utf8_lossy(&tag).to_string()));
                        buffer.clear();
                    },
                    None => {
                        buffer.extend(b"[([[");
                    },
                }
            },
            (Some(x), Some(b'['), Some(b'('), Some(b'[')) => {
                buffer.push(*x);
                cursor += 1;
            },
            (Some(x), Some(y), Some(b'['), Some(b'(')) => {
                buffer.push(*x);
                buffer.push(*y);
                cursor += 2;
            },
            (Some(x), Some(y), Some(z), Some(b'[')) => {
                buffer.push(*x);
                buffer.push(*y);
                buffer.push(*z);
                cursor += 3;
            },
            (Some(x), Some(y), Some(z), Some(w)) => {
                buffer.push(*x);
                buffer.push(*y);
                buffer.push(*z);
                buffer.push(*w);
                cursor += 4;
            },
            (Some(x), _, _, _) => {
                buffer.push(*x);
                cursor += 1;
            },
            (None, _, _, _) => break,
        }
    }

    if !buffer.is_empty() {
        tokens.push(Token::Content(String::from_utf8_lossy(&buffer).to_string()));
    }

    tokens = tokens.into_iter().map(
        |token| match token {
            Token::Content(b) => match limit {
                Some(limit) if b.chars().count() > limit => Token::Content(escape_html(
                    &format!("{}...", b.chars().take(limit.max(3) - 3).collect::<String>())
                )),
                _ => Token::Content(escape_html(&b)),
            },
            t @ Token::Tag(_) => t,
        }
    ).collect();

    let mut cursor = 0;
    let mut result = vec![];

    loop {
        match (
            tokens.get(cursor),
            tokens.get(cursor + 1),
            tokens.get(cursor + 2),
        ) {
            (
                Some(Token::Tag(open)),
                Some(Token::Content(content)),
                Some(Token::Tag(close)),
            ) if open == "segment" && close == "/segment" => {
                result.push(format!(r#"<span class="segment">{content}</span>"#));
                cursor += 3;
            },
            (Some(Token::Tag(tag)), _, _) => {
                cursor += 1;

                let r = match tag.as_str() {
                    "red-flag" => r#"<span class="small-circle red-bg"></span>"#,
                    "green-flag" => r#"<span class="small-circle green-bg"></span>"#,
                    "blue-flag" => r#"<span class="small-circle blue-bg"></span>"#,
                    _ => panic!("TODO: {tag:?}"),
                };
                result.push(r.to_string());
            },
            (Some(Token::Content(content)), _, _) => {
                cursor += 1;
                result.push(content.to_string());
            },
            (None, _, _) => break,
        }
    }

    result.concat()
}

fn parse_tag_name(bytes: &[u8], cursor: usize) -> Option<(&[u8], usize)> {
    match bytes.get(cursor) {
        Some(b'/') => match bytes.get(cursor + 1) {
            Some(b's') => match bytes.get(cursor + 2) {
                Some(b'e') => match bytes.get((cursor + 3)..(cursor + 12)) {
                    Some(b"gment]])]") => Some((&bytes[cursor..(cursor + 8)], cursor + 12)),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        },
        Some(b'b') => match bytes.get(cursor + 1) {
            Some(b'l') => match bytes.get(cursor + 2) {
                Some(b'a') => match bytes.get((cursor + 3)..(cursor + 14)) {
                    Some(b"ck-flag]])]") => Some((&bytes[cursor..(cursor + 10)], cursor + 14)),
                    _ => None,
                },
                Some(b'u') => match bytes.get((cursor + 3)..(cursor + 12)) {
                    Some(b"e-flag]])]") => Some((&bytes[cursor..(cursor + 8)], cursor + 12)),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        },
        Some(b'g') => match bytes.get(cursor + 1) {
            Some(b'r') => match bytes.get(cursor + 2) {
                Some(b'e') => match bytes.get((cursor + 3)..(cursor + 14)) {
                    Some(b"en-flag]])]") => Some((&bytes[cursor..(cursor + 10)], cursor + 14)),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        },
        Some(b'r') => match bytes.get(cursor + 1) {
            Some(b'e') => match bytes.get(cursor + 2) {
                Some(b'd') => match bytes.get((cursor + 3)..(cursor + 12)) {
                    Some(b"-flag]])]") => Some((&bytes[cursor..(cursor + 8)], cursor + 12)),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        },
        Some(b's') => match bytes.get(cursor + 1) {
            Some(b'e') => match bytes.get(cursor + 2) {
                Some(b'g') => match bytes.get((cursor + 3)..(cursor + 11)) {
                    Some(b"ment]])]") => Some((&bytes[cursor..(cursor + 7)], cursor + 11)),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        },
        Some(b'w') => match bytes.get(cursor + 1) { _ => todo!() },
        Some(b'y') => match bytes.get(cursor + 1) { _ => todo!() },
        _ => None,
    }
}

