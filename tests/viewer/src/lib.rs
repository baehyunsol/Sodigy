use serde::{Deserialize, Serialize};
use sodigy_fs_api::{FileError, WriteMode, join, write_string};
use std::collections::hash_map::Entry;
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
use utils::{escape_html, fold_if_long};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Id(u64);

impl fmt::Display for Id {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{:x}", self.0)
    }
}

pub fn render_all(nodes: &[Node], root_dir: &str) -> Result<(), FileError> {
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
        write_string(
            &join(root_dir, &format!("{}.html", node.id))?,
            &wrap_body(&body),
            WriteMode::CreateOrTruncate,
        )?;
    }

    Ok(())
}

// It returns an html (only the body).
fn render(node: &Node, session: &Session) -> String {
    let title = format!(
        r#"<h1>{}{}</h1>"#,
        node.title,
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
        Some(children) => format!(
            r#"<ol>{}</ol>"#,
            children.iter().map(
                |(title, child)| {
                    let (_, flag) = session.title.get(child).unwrap();
                    let flag = if let Some(flag) = flag {
                        format!(r#"<span class="{}"></span>"#, flag.small_flag())
                    } else {
                        String::new()
                    };

                    let grandchildren = session.children.get(child).unwrap();
                    let grandchildren = if let Some(grandchildren) = grandchildren {
                        let s = format!(
                            r#"<ol>{}</ol>"#,
                            grandchildren.iter().map(
                                |(title, grandchild)| {
                                    let (_, flag) = session.title.get(grandchild).unwrap();
                                    let flag = if let Some(flag) = flag {
                                        format!(r#"<span class="{}"></span>"#, flag.small_flag())
                                    } else {
                                        String::new()
                                    };
                                    format!(r#"<li><a href="{grandchild}".html>{title}</a>{flag}</li>"#)
                                }
                            ).collect::<Vec<_>>().concat(),
                        );
                        fold_if_long(s, grandchildren.len())
                    } else {
                        String::new()
                    };

                    format!(r#"<li><a href="{child}.html">{title}</a>{flag}{grandchildren}</li>"#)
                }
            ).collect::<Vec<_>>().concat(),
        ),
        None => String::new(),
    };
    let children = fold_if_long(children, node.children.as_ref().map(|children| children.len()).unwrap_or(0));
    let children = format!(r#"<div class="children">{children}</div>"#);

    let sidebar = match session.parent.get(&node.id) {
        Some(parents) if !parents.is_empty() => todo!(),
        _ => String::new(),
    };

    [title, metadata, children, sidebar].join("\n")
}

fn wrap_body(body: &str) -> String {
    format!(r#"
<html>
<head>
</head>
<body>{body}</body>
</html>
"#)
}
