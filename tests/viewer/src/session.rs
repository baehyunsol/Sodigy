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

    pub leaves: HashMap<Id, Vec<(Vec<String>, Option<Color>)>>,
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
        if let Some(limit) = limit {
            if title.chars().count() > limit {
                escape_html(&format!("{}...", title.chars().take(limit.max(3) - 3).collect::<String>()))
            } else {
                escape_html(title)
            }
        } else {
            escape_html(title)
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

