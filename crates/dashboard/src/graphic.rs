use crate::{Color, Id, escape_html};

#[derive(Clone, Debug)]
pub enum Graphic {
    Title(String),
    TextBox(Vec<(Option<Color>, String)>),
    Text(String),
    List(Vec<Graphic>),
    Link { text: String, id: Id },
    Line,
    // TODO: Image
}

impl Graphic {
    pub fn render(&self, li: bool) -> String {
        match self {
            Graphic::Title(t) => format!("<h2>{}</h2>", escape_html(t)),
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
            Graphic::Text(s) => if li {
                escape_html(s)
            } else {
                format!(r#"<p>{}</p>"#, escape_html(s))
            },
            Graphic::List(e) => format!(
                r#"<ul>{}</ul>"#,
                e.iter().map(
                    |e| format!("<li>{}</li>", e.render(true))
                ).collect::<Vec<_>>().concat(),
            ),
            Graphic::Link { text, id } => format!(r#"<a href="{id}.html">{}</a>"#, escape_html(text)),
            Graphic::Line => String::from("<hr/>"),
        }
    }
}

