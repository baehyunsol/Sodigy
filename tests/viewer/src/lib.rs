use serde::{Deserialize, Serialize};
use sodigy_fs_api::{
    FileError,
    WriteMode,
    create_dir,
    exists,
    join,
    write_string,
};
use std::collections::hash_map::{Entry, HashMap};
use std::fmt;

mod color;
mod graphic;
mod node;
mod session;
mod utils;

pub use color::Color;
pub use graphic::Graphic;
pub use node::Node;
use session::Session;
use utils::escape_html;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Id(u64);

impl Id {
    pub fn init() -> Self {
        Id(4096)
    }

    pub fn increment(&mut self) {
        *self = Id(self.0 + 1);
    }
}

impl fmt::Display for Id {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{:x}", self.0)
    }
}

pub fn render_all(
    nodes: &[Node],
    index_id: Id,
    root_dir: &str,
) -> Result<(), FileError> {
    if !exists(root_dir) {
        create_dir(root_dir)?;
    }

    let mut session = Session::new();

    for node in nodes.iter() {
        session.title.insert(node.id, (node.title.clone(), node.flag));
        session.children.insert(node.id, node.children.clone());

        if let Some(children) = &node.children {
            for (i, (_, child)) in children.iter().enumerate() {
                match session.parent.entry(*child) {
                    Entry::Occupied(mut e) => {
                        e.get_mut().push((node.id, i));
                    },
                    Entry::Vacant(e) => {
                        e.insert(vec![(node.id, i)]);
                    },
                }
            }
        }
    }

    for node in nodes.iter() {
        let body = render(node, &session);
        let html = wrap_body(&node.title, &body);
        write_string(
            &join(root_dir, &format!("{}.html", node.id))?,
            &html,
            WriteMode::CreateOrTruncate,
        )?;

        if node.id == index_id {
            write_string(
                &join(root_dir, "index.html")?,
                &html,
                WriteMode::CreateOrTruncate,
            )?;
        }
    }

    Ok(())
}

// It returns an html (only the body).
fn render(node: &Node, session: &Session) -> String {
    let title = format!(
        r#"<h1>{}{}</h1>"#,
        session.render_title(&node.title, session.config.main_title_length_limit),
        if let Some(flag) = node.flag {
            format!(r#"<span class="{}"></span>"#, flag.big_flag())
        } else {
            String::new()
        },
    );
    let metadata = match &node.metadata {
        Some(data) => format!(r#"<div class="metadata">{}</div>"#, data.iter().map(|g| g.render()).collect::<Vec<_>>().concat()),
        None => String::new(),
    };

    let children = match &node.children {
        Some(children) => {
            let mut children_by_filter = HashMap::new();

            for filter in Color::all_filters() {
                let mut shown = false;
                let s = format!(
                    r#"<ol>{}</ol>"#,
                    children.iter().map(
                        |(title, child)| {
                            let mut c_shown = false;
                            let grandchildren = session.children.get(child).unwrap();
                            let grandchildren = if let Some(grandchildren) = grandchildren {
                                let shown_grandchildren = grandchildren.iter().filter(
                                    |(_, grandchild)| {
                                        let g_shown = session.check_filter(filter, *grandchild);
                                        c_shown |= g_shown;
                                        g_shown
                                    }
                                ).map(
                                    |(title, grandchild)| {
                                        let grandchild = session.render_entry(title, *grandchild);
                                        format!(r#"<li>{grandchild}</li>"#)
                                    }
                                ).collect::<Vec<_>>();
                                let shown_grandchildren_len = shown_grandchildren.len();
                                let shown_grandchildren = shown_grandchildren.concat();

                                let s = if shown_grandchildren_len > 0 {
                                    format!(r#"<ol>{shown_grandchildren}</ol>"#)
                                } else {
                                    String::new()
                                };
                                session.fold_if_long(s, shown_grandchildren_len)
                            } else {
                                String::new()
                            };
                            c_shown |= session.check_filter(filter, *child);
                            shown |= c_shown;

                            if c_shown {
                                format!(r#"<li>{}{grandchildren}</li>"#, session.render_entry(title, *child))
                            } else {
                                String::new()
                            }
                        }
                    ).collect::<Vec<_>>().concat(),
                );

                if shown || filter.is_none() {
                    children_by_filter.insert(filter, s);
                }
            }

            if children_by_filter.len() == 1 {
                format!(r#"<div class="children">{}</div>"#, children_by_filter.values().next().unwrap())
            } else {
                let mut valid_filters: Vec<_> = children_by_filter.keys().collect();
                valid_filters.sort();
                let selectors = valid_filters.iter().enumerate().map(
                    |(i, filter)| {
                        let title = match filter {
                            Some(filter) => format!(r#"<span class="{}"></span>"#, filter.small_flag()),
                            None => String::from("all"),
                        };
                        let checked = if filter.is_none() { " checked" } else { "" };
                        format!(r#"<label class="flag-filter"><input type="radio" name="flag-filter" value="children-{i}"{checked}>{title}</label>"#)
                    }
                ).collect::<Vec<_>>().concat();
                let divs = valid_filters.iter().enumerate().map(
                    |(i, filter)| {
                        let children = children_by_filter.get(filter).unwrap();
                        let hidden = if filter.is_none() { "" } else { " hidden" };
                        format!(r#"<div id="children-{i}" class="children"{hidden}>{children}</div>"#)
                    }
                ).collect::<Vec<_>>().concat();
                let script = r#"
<script>
const radios = document.querySelectorAll('input[name="flag-filter"]');

radios.forEach((radio) => {
    radio.addEventListener("change", () => {
        radios.forEach((r) => {
            document.getElementById(r.value).hidden = !r.checked;
        });
    });
});
</script>
"#;
                format!("{selectors}{divs}{script}")
            }
        },
        None => String::new(),
    };
    let children = format!(r#"<div class="children">{children}</div>"#);

    let sidebar = match session.parent.get(&node.id) {
        Some(parents) if !parents.is_empty() => {
            let mut parents: Vec<String> = parents.iter().map(
                |(parent, i)| {
                    let siblings = session.children.get(parent).unwrap().as_ref().unwrap();
                    let siblings = siblings.iter().enumerate().map(
                        |(j, (title, sibling))| {
                            let (open, close) = if *i == j {
                                ("&gt;&gt;&gt; ", " &lt;&lt;&lt;")
                            } else {
                                ("", "")
                            };
                            format!(r#"<li>{open}{}{close}</li>"#, session.render_entry(title, *sibling))
                        }
                    ).collect::<Vec<_>>().concat();
                    let title = session.render_entry(&session.title.get(parent).unwrap().0, *parent);
                    format!(r#"<div class="parent-name"><h3>{title}</h3></div><div class="parent-content"><ol>{siblings}</ol></div>"#)
                }
            ).collect();
            let sidebar = if parents.len() == 1 {
                parents.pop().unwrap()
            } else {
                parents.iter().map(
                    |parent| format!(r#"<div class="parent-selector">???</div>{parent}"#)
                ).collect::<Vec<_>>().concat()
            };
            format!(r#"
<div class="sidebar"><button class="expand">&lt;&lt;</button>{sidebar}</div>
<script>
const button = document.querySelector(".expand");

button.addEventListener("click", () => {{
    const expanded = document.querySelector(".sidebar").classList.toggle("expanded");
    button.textContent = expanded ? ">>" : "<<";
}});
</script>
"#)
        },
        _ => String::new(),
    };

    [title, metadata, children, sidebar].join("\n")
}

const STYLE: &str = include_str!("../style.css");

fn wrap_body(title: &str, body: &str) -> String {
    format!(r#"
<!DOCTYPE html>
<html>
<head>
<style>
{STYLE}
</style>

<title>{title}</title>
</head>
<body>{body}</body>
</html>
"#)
}

