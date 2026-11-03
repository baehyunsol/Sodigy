use crate::{Color, escape_html};
use serde::{Deserialize, Serialize};

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

