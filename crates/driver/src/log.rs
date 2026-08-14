use sodigy_error::{Error, dump_errors};
use sodigy_fs_api::{
    FileError,
    WriteMode,
    create_dir_all,
    exists,
    join,
    write_string,
};
use sodigy_inter_mir::{Session as InterMirSession, TypeError};
use sodigy_mir::Type;
use sodigy_prettify::prettify;
use sodigy_span::{
    RenderSpanOption,
    RenderSpanSession,
    RenderableSpan,
    render_spans,
};
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

// VIBE NOTE: many css and javascript in this function are written by AI.
fn render_page_and_save(
    parent: Option<usize>,
    calls: &[FuncCall],
    index: usize,
    preview_map: &HashMap<usize, PreviewMap>,
    log_dir: &str,
    intermediate_dir: &str,
    render_span_option: &RenderSpanOption,
    render_span_session: &mut RenderSpanSession,
) -> Result<(), FileError> {
    let call = &calls[index];
    let call_index = call.call_index;
    let preview_map_rendered = render_preview_map(preview_map, &call.children, call_index);

    let first_index = if index != 0 && let Some(call) = calls.get(0) { Some(call.call_index) } else { None };
    let prev_index = if index > 0 { Some(calls[index - 1].call_index) } else { None };
    let next_index = if let Some(call) = calls.get(index + 1) { Some(call.call_index) } else { None };
    let last_index = if index != calls.len() - 1 && let Some(call) = calls.last() { Some(call.call_index) } else { None };

    fn create_button(title: &str, index: Option<usize>) -> String {
        if let Some(index) = index {
            format!(r#"<a href="../{:02}/{index}.html">{title}</a>"#, index % 100)
        } else {
            title.to_string()
        }
    }

    let page = format!("{}/{}", index + 1, calls.len());
    let page = format!("{}{page}{}", " ".repeat((13 - page.len()) / 2), " ".repeat((13 - page.len()) / 2));
    let buttons = format!(
        "                      {}\n\n{} {}{page}{} {}\n\n                     {}",
        create_button("up", parent),
        create_button("&lt;&lt;&lt; first", first_index),
        create_button("&lt;&lt; prev", prev_index),
        create_button("next &gt;&gt;", next_index),
        create_button("last &gt;&gt;&gt;", last_index),
        create_button("down", call.children.get(0).map(|c| c.call_index)),
    );

    for (i, _) in call.children.iter().enumerate() {
        render_page_and_save(
            Some(call_index),
            &call.children,
            i,
            preview_map,
            log_dir,
            intermediate_dir,
            render_span_option,
            render_span_session,
        )?;
    }

    let spans = render_spans(
        &call.spans,
        render_span_option,
        render_span_session,
    );
    let spans = escape_html(&spans);
    let title = call.title();

    let input = call.input.iter().enumerate().map(
        |(i, input)| format!("<li>{}</li>", input.render(i))
    ).collect::<Vec<_>>().concat();
    let output = call.output.iter().enumerate().map(
        |(i, output)| format!("<li>{}</li>", output.render(i + 1000))
    ).collect::<Vec<_>>().concat();
    let error = if call.has_error {
        let error = call.last_errors.iter().enumerate().map(
            |(i, (type_error, error))| {
                let type_error_str = format!("{type_error:?}");

                let value = Value {
                    name: String::from("e"),
                    short: format!("{}...", type_error_str.chars().take(40).collect::<String>()),
                    long: Some(vec![
                        dump_errors(
                            vec![error.clone()],
                            vec![],
                            intermediate_dir,
                            Default::default(),
                            None,
                            false,
                        ),
                        String::from_utf8(prettify(type_error_str.into_bytes())).unwrap(),
                        String::from_utf8(prettify(format!("{error:?}").into_bytes())).unwrap(),
                    ].join("\n\n------------\n\n")),
                };

                format!("<li>{}</li>", value.render(i + 2000))
            }
        ).collect::<Vec<_>>().concat();
        format!(r#"
<li>error<ul>{error}</ul></li>
"#)
    } else {
        String::new()
    };

    let body = format!(r#"
<h1>{title}</h1>

{preview_map_rendered}

<pre>
<code>
{buttons}
</code>
</pre>

<pre class="code-block">
<code>
{spans}
</code>
</pre>

<ul>
    <li>input<ul>{input}</ul></li>
    <li>output<ul>{output}</ul></li>
    {error}
</ul>
"#);
    let inter_dir = join(log_dir, &format!("{:02}", call_index % 100))?;

    if !exists(&inter_dir) {
        create_dir_all(&inter_dir)?;
    }

    let path = join(&inter_dir, &format!("{call_index}.html"))?;
    write_string(&path, &to_html(&call_index.to_string(), &body), WriteMode::CreateOrTruncate)?;
    Ok(())
}

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

fn render_preview_map(preview_map: &HashMap<usize, PreviewMap>, children: &[FuncCall], call_index: usize) -> String {
    fn render_single_preview(entries: &[(String, usize, bool)], cursor: Option<usize>) -> String {
        let list = entries.iter().enumerate().map(
            |(i, (title, call_index, has_error))| format!(
                r#"<li>{}<a href="../{:02}/{call_index}.html">{title}</a>{}{}</li>"#,
                if cursor == Some(i) { "&gt;&gt;&gt; " } else { "   " },
                call_index % 100,
                if *has_error {
                    r#" <span class="error-marker">E</span>"#
                } else {
                    ""
                },
                if cursor == Some(i) { " &lt;&lt;&lt;" } else { "   " },
            )
        ).collect::<Vec<_>>().concat();

        format!(r#"
<div class="preview-map">
    <ol>
        {list}
    </ol>
</div>
"#)
    }

    let mut preview = preview_map.get(&call_index).unwrap();
    let mut previews = vec![];
    previews.push(render_single_preview(
        &children.iter().map(
            |c| (c.title(), c.call_index, c.has_error)
        ).collect::<Vec<_>>(),
        None,
    ));

    for _ in 0..4 {
        previews.push(render_single_preview(&preview.entries, Some(preview.cursor)));

        if let Some(parent) = preview.parent {
            preview = preview_map.get(&parent).unwrap();
        }

        else {
            break;
        }
    }

    format!(
        r#"<div class="preview-map-box">{}</div>"#,
        previews.into_iter().rev().collect::<Vec<_>>().join("--&gt;"),
    )
}

fn render_map_and_save(
    calls: &[FuncCall],
    log_dir: &str,
) -> Result<(), FileError> {
    fn render_map(calls: &[FuncCall], error_only: bool, recursion_limit: usize) -> String {
        if recursion_limit == 0 || calls.is_empty() {
            String::new()
        }

        else {
            format!(
                "<ol>{}</ol>",
                calls.iter().filter(
                    |call| !error_only || call.has_inner_error
                ).map(
                    |call| format!(
                        r#"<li><a href="../{:02}/{}.html">{}</a>{}{}</li>"#,
                        call.call_index % 100,
                        call.call_index,
                        call.title(),
                        if call.has_error {
                            r#" <span class="error-marker">E</span>"#
                        } else {
                            ""
                        },
                        render_map(&call.children, error_only, recursion_limit - 1),
                    )
                ).collect::<Vec<_>>().concat(),
            )
        }
    }

    let inter_dir = join(log_dir, "indexes")?;

    if !exists(&inter_dir) {
        create_dir_all(&inter_dir)?;
    }

    write_string(
        &join(&inter_dir, "map.html")?,
        &to_html("map", &render_map(calls, false, 4)),
        WriteMode::CreateOrTruncate,
    )?;

    write_string(
        &join(&inter_dir, "error.html")?,
        &to_html("error", &render_map(calls, true, 12)),
        WriteMode::CreateOrTruncate,
    )?;

    Ok(())
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
