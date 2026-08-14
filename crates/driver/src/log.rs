use sodigy_error::Error;
use sodigy_inter_mir::{Session as InterMirSession, TypeError};
use sodigy_mir::Type;
use sodigy_prettify::prettify;
use sodigy_span::RenderableSpan;
use std::collections::HashMap;

mod inter_hir;
mod inter_mir;
mod post_mir;

pub use inter_hir::dump_inter_hir_log;
pub use inter_mir::dump_inter_mir_log;
pub use post_mir::dump_post_mir_log;

const STYLE: &str = include_str!("log/style.css");

fn to_html(title: &str, body: &str) -> String {
    format!(r#"
<!DOCTYPE html>
<html>

<head>
<style>
{STYLE}
</style>

<title>{title}</title>
</head>

<body>

<a href="../00/0.html">Home</a>
<a href="../indexes/map.html">Map</a>
<a href="../indexes/error.html">Error</a>

{body}
</body>

</html>
"#)}

struct FuncCall {
    call_index: usize,
    name: String,
    input: Vec<Value>,
    children: Vec<FuncCall>,
    output: Vec<Value>,
    spans: Vec<RenderableSpan>,
    has_error: bool,
    last_errors: Vec<(Option<TypeError>, Error)>,

    // self.has_error || self.children.any(|c| c.has_inner_error)
    has_inner_error: bool,
}

impl FuncCall {
    fn title(&self) -> String {
        format!(
            "{}({}) -> ({})",
            self.name,
            self.input.iter().map(|c| format!("{}={}", c.name, c.short)).collect::<Vec<_>>().join(", "),
            self.output.iter().map(|c| format!("{}={}", c.name, c.short)).collect::<Vec<_>>().join(", "),
        )
    }
}

struct Value {
    name: String,
    short: String,
    long: Option<String>,
}

impl Value {
    pub fn render(&self, global_id: usize) -> String {
        let name = &self.name;

        format!(
            r#"{name}: <span class="code-span"><code>{}</code></span>{}"#,
            self.short,
            if let Some(long) = &self.long {
                let button = format!(r#"<span class="modal-button" id="button-{global_id}">(i)</span>"#);
                let modal = format!(r#"<div class="modal-box hidden" id="m-{global_id}"><div class="modal-content" id="m-c-{global_id}"><pre class="code-block"><code>{}</code></pre></div></div>"#, escape_html(long));
                let script = format!(r##"
<script>
// Add a close button to each modal automatically
var modal_{global_id} = document.getElementById(`m-c-{global_id}`);
var closeButton = document.createElement("button");
closeButton.type = "button";
closeButton.className = "modal-close";
closeButton.innerHTML = "&times;";
closeButton.setAttribute("aria-label", "Close");

modal_{global_id}.prepend(closeButton);

var button_{global_id} = document.getElementById(`button-{global_id}`);
button_{global_id}.addEventListener("click", () => {{
    const modal = document.getElementById(`m-{global_id}`);

    if (modal) {{
        modal.classList.remove("hidden");
    }}
}});

document.addEventListener("click", (event) => {{
var closeButton = event.target.closest(".modal-close");

if (closeButton) {{
    closeButton.closest(".modal-box").classList.add("hidden");
}}
}});
</script>
"##);
                format!(" {button}{modal}{script}")
            } else {
                String::new()
            },
        )
    }

    pub fn from_optional_type(name: &str, r#type: &Option<Type>, session: &InterMirSession) -> Value {
        match r#type {
            Some(r#type) => Value {
                name: name.to_string(),
                short: escape_html(&session.render_type(r#type)),
                long: Some(String::from_utf8(prettify(format!("{type:?}").into_bytes())).unwrap()),
            },
            None => Value {
                name: name.to_string(),
                short: String::from("N/A"),
                long: None,
            },
        }
    }
}

struct PreviewMap {
    entries: Vec<(String, usize, bool)>,
    cursor: usize,
    parent: Option<usize>,
}

fn create_preview_map(calls: &[FuncCall]) -> HashMap<usize, PreviewMap> {
    fn create_preview_map_worker(calls: &[FuncCall], parent: Option<usize>, result: &mut HashMap<usize, PreviewMap>) {
        let entries: Vec<_> = calls.iter().map(
            |call| (call.title(), call.call_index, call.has_error)
        ).collect();

        for (i, entry) in calls.iter().enumerate() {
            create_preview_map_worker(&entry.children, Some(entry.call_index), result);

            result.insert(
                entry.call_index,
                PreviewMap {
                    entries: entries.clone(),
                    cursor: i,
                    parent,
                },
            );
        }
    }

    let mut result = HashMap::new();
    create_preview_map_worker(calls, None, &mut result);
    result
}

fn escape_html(s: &str) -> String {
    let s = s
        .replace("&", "&amp;")
        .replace(">", "&gt;")
        .replace("<", "&lt;");

    apply_ansi_term_color(&s)
}

#[derive(Clone, Copy)]
enum TermColorParseState {
    Text,
    Control,
}

fn apply_ansi_term_color(s: &str) -> String {
    let mut state = TermColorParseState::Text;
    let mut content_buffer: Vec<char> = vec![];
    let mut digits_buffer: Vec<char> = vec![];
    let mut result: Vec<String> = vec![String::from("<span>")];

    for ch in s.chars() {
        match state {
            TermColorParseState::Text => match ch {
                '\u{1b}' => {
                    digits_buffer = vec![];
                    result.push(content_buffer.drain(..).collect());
                    result.push(String::from("</span>"));
                    state = TermColorParseState::Control;
                },
                _ => {
                    content_buffer.push(ch);
                },
            },
            TermColorParseState::Control => match ch {
                '0'..='9' => {
                    digits_buffer.push(ch);
                },
                'm' => {
                    match digits_buffer.iter().collect::<String>().parse::<u32>() {
                        Ok(0) => {
                            result.push(String::from(r#"<span>"#));
                        },
                        Ok(31) => {
                            result.push(String::from(r#"<span class="red">"#));
                        },
                        Ok(32) => {
                            result.push(String::from(r#"<span class="green">"#));
                        },
                        Ok(33) => {
                            result.push(String::from(r#"<span class="yellow">"#));
                        },
                        Ok(34) => {
                            result.push(String::from(r#"<span class="blue">"#));
                        },
                        _ => unreachable!(),
                    };

                    state = TermColorParseState::Text;
                },
                _ => {},
            },
        }
    }

    if !content_buffer.is_empty() {
        result.push(content_buffer.drain(..).collect());
        result.push(String::from("</span>"));
    }

    result.concat()
}
