use serde::{Deserialize, Serialize};
use sodigy_fs_api::{FileError, WriteMode, join, write_string};
use std::collections::hash_map::{Entry, HashMap};
use std::fmt;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Node {
    pub id: Id,
    pub title: String,
    pub flag: Option<Color>,
    pub metadata: Option<Vec<Graphic>>,
    pub children: Option<Vec<(String, Id)>>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Color {
    Red,
    Green,
    Blue,
    Yellow,
}

impl Color {
    pub fn big_flag(&self) -> String {
        format!("big {self:?}").to_ascii_lowercase()
    }

    pub fn small_flag(&self) -> String {
        format!("small {self:?}").to_ascii_lowercase()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Id(u64);

impl fmt::Display for Id {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{:x}", self.0)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Graphic {
    TextBox(Vec<(Option<Color>, String)>),
    Text(String),
    List(Vec<Graphic>),
}

impl Graphic {
    pub fn render(&self) -> String {
        match self {
            Graphic::TextBox(s) => format!(
                r#"<pre class="text-box"><code>{}</code></pre>"#,
                s.iter().map(
                    |(color, content)| if let Some(color) = color {
                        format!(
                            r#"<span class="{}">{}</span>"#,
                            format!("{color:?}").to_ascii_lowercase(),
                            escape_html(content),
                        )
                    } else {
                        escape_html(content)
                    }
                ).collect::<Vec<_>>().concat(),
            ),
            Graphic::Text(s) => format!(r#"<p>{}</p>"#, escape_html(s)),
            Graphic::List(e) => format!(
                r#"<ul>{}</ul>"#,
                e.iter().map(
                    |e| e.render()
                ).collect::<Vec<_>>().concat(),
            ),
        }
    }
}

struct Session {
    // These data have to be collected before any nodes are rendered.
    pub title: HashMap<Id, (String, Option<Color>)>,
    pub children: HashMap<Id, Option<Vec<(String, Id)>>>,
    pub parent: HashMap<Id, Vec<(Id, usize)>>,
}

impl Session {
    pub fn new() -> Session {
        Session {
            title: HashMap::new(),
            children: HashMap::new(),
            parent: HashMap::new(),
        }
    }
}

fn render_all(nodes: &[Node], root_dir: &str) -> Result<(), FileError> {
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

fn fold_if_long(html: String, len: usize) -> String {
    if len > 5 {
        todo!()
    } else {
        html
    }
}

fn escape_html(s: &str) -> String {
    s
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
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

