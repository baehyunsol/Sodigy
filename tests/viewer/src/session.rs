use crate::{Color, Id};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize)]
pub struct Session {
    // These data have to be collected before any nodes are rendered.
    pub title: HashMap<Id, (String, Option<Color>)>,
    pub children: HashMap<Id, Option<Vec<(String, Id)>>>,
    pub parent: HashMap<Id, Vec<(Id, usize)>>,

    pub fold_threshold: usize,
}

impl Session {
    pub fn new() -> Session {
        Session {
            title: HashMap::new(),
            children: HashMap::new(),
            parent: HashMap::new(),
            fold_threshold: 5,
        }
    }

    pub fn fold_if_long(&self, html: String, len: usize) -> String {
        if len > self.fold_threshold {
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
        format!(r#"<a href="{id}.html">{title}</a>{flag}"#)
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

