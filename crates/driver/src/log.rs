use sodigy_fs_api::{
    FileError,
    WriteMode,
    create_dir,
    create_dir_all,
    exists,
    join,
    join3,
    join4,
    parent,
    write_bytes,
    write_string,
};
use sodigy_inter_mir::{LogEntry, Session as InterMirSession, TypeError};
use sodigy_parse::merge_field_spans;
use sodigy_post_mir::MatchDump;
use sodigy_prettify::prettify;
use sodigy_span::{
    RenderSpanOption,
    RenderSpanSession,
    RenderableSpan,
    render_spans,
};
use sodigy_string::InternedString;
use std::collections::hash_map::{Entry, HashMap};

pub fn log_matches(matches: &Vec<MatchDump>, intermediate_dir: &str) -> Result<(), FileError> {
    let mut buffer = vec![];
    let mut render_span_session = RenderSpanSession::new(intermediate_dir);
    let render_span_option = RenderSpanOption {
        max_width: 128,
        max_height: 20,
        context: 3,
        render_source: true,
        color: None,
        group_delim: None,
    };

    for MatchDump { keyword_span, span_helpers, decision_tree, expr } in matches.iter() {
        let mut spans = keyword_span.simple_error();

        for (span, helper) in span_helpers.iter() {
            spans.push(RenderableSpan {
                span: span.clone(),
                auxiliary: true,
                note: Some(helper.to_string()),
            });
        }

        buffer.push(String::from("------\n"));
        buffer.push(String::from("# Sodigy\n"));
        buffer.push(String::from("```\n"));
        buffer.push(render_spans(
            &spans,
            &render_span_option,
            &mut render_span_session,
        ));
        buffer.push(String::from("```\n"));
        buffer.push(String::new());
        buffer.push(String::from("# Decision Tree\n"));
        buffer.push(String::from("```\n"));
        buffer.push(decision_tree.to_string());
        buffer.push(String::from("```\n"));
        buffer.push(String::new());
        buffer.push(String::from("# Expr\n"));
        buffer.push(String::from("```\n"));
        buffer.push(expr.to_string());
        buffer.push(String::from("```\n"));
    }

    let save_at = join4(
        intermediate_dir,
        "irs",
        "postmir",
        "log",
    )?;

    if !exists(&parent(&save_at)?) {
        create_dir(&parent(&save_at)?)?;
    }

    if !exists(&save_at) {
        write_bytes(&save_at, b"", WriteMode::AlwaysCreate)?;
    }

    write_string(
        &save_at,
        &buffer.join("\n"),
        WriteMode::AlwaysAppend,
    )
}


const STYLE: &str = "
body {
background-color: rgb(48, 48, 48);
color: rgb(224, 244, 244);
}

.code-block {
padding: 8px;
background-color: rgb(0, 0, 0);
color: rgb(255, 255, 255);
overflow: auto;
}

.code-span {
padding: 4px;
background-color: rgb(0, 0, 0);
color: rgb(192, 128, 0);
border-radius: 8px;
white-space: pre;
}

.hidden {
display: none;
}

span.modal-button {
cursor: pointer;
background-color: rgb(255, 255, 255);
color: rgb(32, 32, 32);
border-radius: 32px;
padding: 4px;
}

.modal-box {
position: fixed;
top: 0;
left: 0;
width: 100%;
height: 100%;
background-color: #2222;
backdrop-filter: blur(5px);
z-index: 1;
}

.modal-content {
position: fixed;
top: 50%;
left: 50%;
transform: translate(-50%, -50%);
max-width: 80vw;
max-height: 80vh;
overflow: auto;
z-index: 2;
}

li {
margin-bottom: 8px;
}

a:link {
border-bottom: 4px solid rgba(64, 192, 255, 0.5);
color: rgb(64, 192, 255);
text-decoration: none;
transition: border 0.3s;
}

a:visited {
color: rgb(64, 192, 255);
text-decoration: none;
}

a:hover {
text-decoration: none;
border-bottom: 4px solid rgb(64, 192, 255);
}

a::selection {
color: rgb(192, 64, 0);
}
";

pub fn log_inter_mir(session: &InterMirSession) -> Result<(), FileError> {
    struct FuncCall {
        call_index: usize,
        name: String,
        input: Vec<Value>,
        children: Vec<FuncCall>,
        output: Vec<Value>,
        spans: Vec<RenderableSpan>,
        has_error: bool,
        last_errors: Vec<TypeError>,
    }

    struct Value {
        name: String,
        short: String,
        long: Option<String>,
    }

    // It's supposed to return `(FuncCall, usize)`, but some are not implemented yet, so it returns `Option<FuncCall>`.
    fn to_func_call(log: &[LogEntry], mut index: usize, session: &InterMirSession) -> (Option<FuncCall>, usize) {
        let mut spans = vec![];
        let mut input = vec![];

        let call_index = index;
        let (name, log_id) = match &log[index] {
            e @ LogEntry::TypeSolveLoopStart(_) => ("type_solve_loop", e.id().unwrap()),
            LogEntry::SolveSupertypeStart { id, lhs, rhs, lhs_span, rhs_span, context } => {
                input.push(Value {
                    name: String::from("lhs"),
                    short: session.render_type(lhs),
                    long: Some(String::from_utf8(prettify(format!("{lhs:?}").into_bytes())).unwrap()),
                });
                input.push(Value {
                    name: String::from("rhs"),
                    short: session.render_type(rhs),
                    long: Some(String::from_utf8(prettify(format!("{rhs:?}").into_bytes())).unwrap()),
                });
                input.push(Value {
                    name: String::from("context"),
                    short: format!("{context:?}"),
                    long: None,
                });

                if let Some(lhs_span) = lhs_span {
                    spans.push(RenderableSpan {
                        span: lhs_span.clone(),
                        auxiliary: false,
                        note: Some(String::from("lhs")),
                    });
                }

                if let Some(rhs_span) = rhs_span {
                    spans.push(RenderableSpan {
                        span: rhs_span.clone(),
                        auxiliary: false,
                        note: Some(String::from("rhs")),
                    });
                }

                ("solve_supertype", *id)
            },
            LogEntry::SolveFuncStart { id, func } => {
                input.push(Value {
                    name: String::from("func"),
                    short: func.name.unintern_or_default(&session.intermediate_dir),
                    long: Some(String::from_utf8(prettify(format!("{func:?}").into_bytes())).unwrap()),
                });

                spans.push(RenderableSpan {
                    span: func.name_span.clone(),
                    auxiliary: false,
                    note: Some(String::from("func")),
                });

                ("solve_func", *id)
            },
            LogEntry::SolveLetStart { id, r#let } => {
                input.push(Value {
                    name: String::from("let"),
                    short: r#let.name.unintern_or_default(&session.intermediate_dir),
                    long: Some(String::from_utf8(prettify(format!("{let:?}").into_bytes())).unwrap()),
                });

                spans.push(RenderableSpan {
                    span: r#let.name_span.clone(),
                    auxiliary: false,
                    note: Some(String::from("let")),
                });

                ("solve_let", *id)
            },
            LogEntry::SolveAssertStart { id, assert } => {
                input.push(Value {
                    name: String::from("assert"),
                    short: assert.name.map(|name| name.unintern_or_default(&session.intermediate_dir)).unwrap_or(String::from("unnamed")),
                    long: Some(String::from_utf8(prettify(format!("{assert:?}").into_bytes())).unwrap()),
                });

                spans.push(RenderableSpan {
                    span: assert.keyword_span.clone(),
                    auxiliary: false,
                    note: Some(String::from("assert")),
                });

                ("solve_assert", *id)
            },
            LogEntry::SolveExprStart { id, expr } => {
                input.push(Value {
                    name: String::from("expr"),
                    short: String::from("(...)"),  // TODO: dump_expr?
                    long: Some(String::from_utf8(prettify(format!("{expr:?}").into_bytes())).unwrap()),
                });

                spans.push(RenderableSpan {
                    span: expr.error_span_wide(),
                    auxiliary: false,
                    note: Some(String::from("expr")),
                });

                ("solve_expr", *id)
            },
            LogEntry::GetTypeOfFieldStart { id, r#type, field } => {
                input.push(Value {
                    name: String::from("type"),
                    short: session.render_type(r#type),
                    long: Some(String::from_utf8(prettify(format!("{type:?}").into_bytes())).unwrap()),
                });

                input.push(Value {
                    name: String::from("field"),
                    short: String::new(),  // TODO: impl `dump_field()` -> it's in `mir::dump_expr`, we have to extract it
                    long: Some(String::from_utf8(prettify(format!("{field:?}").into_bytes())).unwrap()),
                });

                spans.push(RenderableSpan {
                    span: merge_field_spans(field),
                    auxiliary: false,
                    note: Some(String::from("field")),
                });

                ("get_type_of_field", *id)
            },
            LogEntry::GetItemShapeStart { id, r#type, def_span } => {
                input.push(Value {
                    name: String::from("type"),
                    short: session.render_type(r#type),
                    long: Some(String::from_utf8(prettify(format!("{type:?}").into_bytes())).unwrap()),
                });
                input.push(Value {
                    name: String::from("def_span"),
                    short: String::from("(...)"),
                    long: Some(String::from_utf8(prettify(format!("{def_span:?}").into_bytes())).unwrap()),
                });

                spans.push(RenderableSpan {
                    span: def_span.clone(),
                    auxiliary: false,
                    note: Some(String::from("def_span")),
                });

                ("get_item_shape", *id)
            },
            _ => return (None, index + 1),
        };

        let mut children = vec![];
        index += 1;

        while log[index].id() != Some(log_id) {
            let (child, new_index) = to_func_call(log, index, session);
            index = new_index;

            if let Some(child) = child {
                children.push(child);
            }
        }

        let mut output = vec![];

        let (has_error, last_errors) = match &log[index] {
            LogEntry::TypeSolveLoopEnd(_) => (false, vec![]),
            LogEntry::SolveSupertypeEnd { solved_type, has_error, last_errors, .. } => {
                match solved_type {
                    Some(t) => {
                        output.push(Value {
                            name: String::from("solved_type"),
                            short: session.render_type(t),
                            long: Some(String::from_utf8(prettify(format!("{t:?}").into_bytes())).unwrap()),
                        });
                    },
                    None => {
                        output.push(Value {
                            name: String::from("solved_type"),
                            short: String::from("N/A"),
                            long: None,
                        });
                    },
                }

                (*has_error, last_errors.clone())
            },
            LogEntry::SolveFuncEnd { annotated_type, infered_type, has_error, last_errors, .. } => {
                output.push(Value {
                    name: String::from("annotated_type"),
                    short: session.render_type(annotated_type),
                    long: Some(String::from_utf8(prettify(format!("{annotated_type:?}").into_bytes())).unwrap()),
                });

                match infered_type {
                    Some(t) => {
                        output.push(Value {
                            name: String::from("infered_type"),
                            short: session.render_type(t),
                            long: Some(String::from_utf8(prettify(format!("{t:?}").into_bytes())).unwrap()),
                        });
                    },
                    None => {
                        output.push(Value {
                            name: String::from("infered_type"),
                            short: String::from("N/A"),
                            long: None,
                        });
                    },
                }

                (*has_error, last_errors.clone())
            },
            LogEntry::SolveLetEnd { annotated_type, infered_type, has_error, last_errors, .. } => {
                output.push(Value {
                    name: String::from("annotated_type"),
                    short: session.render_type(annotated_type),
                    long: Some(String::from_utf8(prettify(format!("{annotated_type:?}").into_bytes())).unwrap()),
                });

                match infered_type {
                    Some(t) => {
                        output.push(Value {
                            name: String::from("infered_type"),
                            short: session.render_type(t),
                            long: Some(String::from_utf8(prettify(format!("{t:?}").into_bytes())).unwrap()),
                        });
                    },
                    None => {
                        output.push(Value {
                            name: String::from("infered_type"),
                            short: String::from("N/A"),
                            long: None,
                        });
                    },
                }

                (*has_error, last_errors.clone())
            },
            LogEntry::SolveAssertEnd { has_error, last_errors, .. } => (*has_error, last_errors.clone()),
            LogEntry::SolveExprEnd { infered_type, has_error, last_errors, .. } => {
                match infered_type {
                    Some(t) => {
                        output.push(Value {
                            name: String::from("infered_type"),
                            short: session.render_type(t),
                            long: Some(String::from_utf8(prettify(format!("{t:?}").into_bytes())).unwrap()),
                        });
                    },
                    None => {
                        output.push(Value {
                            name: String::from("infered_type"),
                            short: String::from("N/A"),
                            long: None,
                        });
                    },
                }

                (*has_error, last_errors.clone())
            },
            LogEntry::GetTypeOfFieldEnd { associated_func, infered_type, has_error, last_errors, .. } => {
                // TODO: dump associated_func
                match infered_type {
                    Some(t) => {
                        output.push(Value {
                            name: String::from("infered_type"),
                            short: session.render_type(t),
                            long: Some(String::from_utf8(prettify(format!("{t:?}").into_bytes())).unwrap()),
                        });
                    },
                    None => {
                        output.push(Value {
                            name: String::from("infered_type"),
                            short: String::from("N/A"),
                            long: None,
                        });
                    },
                }

                (*has_error, last_errors.clone())
            },
            LogEntry::GetItemShapeEnd { struct_shape, enum_shape, .. } => {
                match struct_shape {
                    Some(s) => {
                        output.push(Value {
                            name: String::from("struct_shape"),
                            short: String::from("..."),
                            long: Some(String::from_utf8(prettify(format!("{s:?}").into_bytes())).unwrap()),
                        });
                    },
                    None => {
                        output.push(Value {
                            name: String::from("struct_shape"),
                            short: String::from("N/A"),
                            long: None,
                        });
                    },
                }

                match enum_shape {
                    Some(s) => {
                        output.push(Value {
                            name: String::from("enum_shape"),
                            short: String::from("..."),
                            long: Some(String::from_utf8(prettify(format!("{s:?}").into_bytes())).unwrap()),
                        });
                    },
                    None => {
                        output.push(Value {
                            name: String::from("enum_shape"),
                            short: String::from("N/A"),
                            long: None,
                        });
                    },
                }

                (false, vec![])
            },
            _ => todo!(),
        };

        (
            Some(FuncCall {
                call_index,
                name: name.to_string(),
                input,
                children,
                output,
                spans,
                has_error,
                last_errors,
            }),
            index + 1,
        )
    }

    let mut index = 0;
    let mut calls = vec![];

    while session.log.get(index).is_some() {
        let (call, new_index) = to_func_call(&session.log, index, &session);
        index = new_index;

        if let Some(call) = call {
            calls.push(call);
        }
    }

    // VIBE NOTE: many css and javascript in this function are written by AI.
    fn render_page_and_save(
        parent: Option<usize>,
        calls: &[FuncCall],
        index: usize,
        log_lines: &[String],
        session: &InterMirSession,
        render_span_option: &RenderSpanOption,
        render_span_session: &mut RenderSpanSession,
    ) -> Result<(), FileError> {
        let call = &calls[index];
        let call_index = call.call_index;
        let mut log_context = Vec::with_capacity(12);
        log_context.push(format!("{call_index}/{}", log_lines.len()));

        if call_index < 5 {
            for _ in 0..(5 - call_index) {
                log_context.push(String::new());
            }
        }

        for i in (call_index.max(5) - 5)..(call_index + 6).min(log_lines.len()) {
            let pointer = if i == call_index { "&gt;&gt;&gt;" } else { "   " };
            log_context.push(format!("{pointer}{}", log_lines[i]));
        }

        let log_context = log_context.join("\n");

        let prev_index = if index > 0 {
            Some(calls[index - 1].call_index)
        } else {
            None
        };

        let next_index = if let Some(call) = calls.get(index + 1) {
            Some(call.call_index)
        } else {
            None
        };

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
            "            {}\n\n{}{page}{}\n\n           {}",
            create_button("up", parent),
            create_button("&lt;&lt; prev", prev_index),
            create_button("next &gt;&gt;", next_index),
            create_button("down", call.children.get(0).map(|c| c.call_index)),
        );

        for (i, child) in call.children.iter().enumerate() {
            render_page_and_save(
                Some(call_index),
                &call.children,
                i,
                log_lines,
                session,
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

        let title = format!(
            "{}({}) -> {}",
            call.name,
            call.input.iter().map(|c| c.name.to_string()).collect::<Vec<_>>().join(", "),
            if call.output.len() == 1 {
                call.output[0].name.to_string()
            } else {
                format!("({})", call.output.iter().map(|c| c.name.to_string()).collect::<Vec<_>>().join(", "))
            },
        );

        fn render_value(value: &Value, id: usize) -> String {
            let name = &value.name;

            format!(
                r#"{name}: <span class="code-span"><code>{}</code></span>{}"#,
                value.short,
                if let Some(long) = &value.long {
                    let button = format!(r#"<span class="modal-button" id="button-{id}">(i)</span>"#);
                    let modal = format!(r#"<div class="modal-box hidden" id="m-{id}"><div class="modal-content" id="m-c-{id}"><pre class="code-block"><code>{}</code></pre></div></div>"#, escape_html(long));
                    let script = format!(r##"
<script>
// Add a close button to each modal automatically
var modal_{id} = document.getElementById(`m-c-{id}`);
var closeButton = document.createElement("button");
closeButton.type = "button";
closeButton.className = "modal-close";
closeButton.innerHTML = "&times;";
closeButton.setAttribute("aria-label", "Close");

modal_{id}.prepend(closeButton);

var button_{id} = document.getElementById(`button-{id}`);
button_{id}.addEventListener("click", () => {{
    const modal = document.getElementById(`m-{id}`);

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

        let input = call.input.iter().enumerate().map(
            |(i, input)| format!("<li>{}</li>", render_value(input, i))
        ).collect::<Vec<_>>().concat();
        let output = call.output.iter().enumerate().map(
            |(i, output)| format!("<li>{}</li>", render_value(output, i + 1000))
        ).collect::<Vec<_>>().concat();

        let s = format!(r#"
<!DOCTYPE html>
<html>

<head>
<style>
{STYLE}
</style>
</head>

<body>

<a href="../00/0.html">Home</a>
<a href="../indexes/func.html">Func</a>

<h1>{title}</h1>

<pre class="code-block">
<code>
{log_context}
</code>
</pre>

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
</ul>
</body>

</html>
"#);
        let inter_dir = join3(
            &session.intermediate_dir,
            "irs",
            &join3(
                "intermir",
                "log",
                &format!("{:02}", call_index % 100),
            )?,
        )?;

        if !exists(&inter_dir) {
            create_dir_all(&inter_dir)?;
        }

        let path = join(&inter_dir, &format!("{call_index}.html"))?;
        write_string(&path, &s, WriteMode::CreateOrTruncate)?;
        Ok(())
    }

    let log_lines = session.log.iter().enumerate().map(
        |(i, log)| {
            let mut log = format!("{log:?}");

            if log.chars().count() > 120 {
                log = format!("{}...", log.chars().take(120).collect::<String>());
            }

            format!(" {i:>5} | {log}")
        }
    ).collect::<Vec<_>>();

    let render_span_option = RenderSpanOption {
        max_height: 16,
        max_width: 120,
        context: 8,
        render_source: true,
        color: None,  // TODO: color spans
        group_delim: None,
    };
    let mut render_span_session = RenderSpanSession::new(&session.intermediate_dir);

    for (i, _) in calls.iter().enumerate() {
        render_page_and_save(
            None,
            &calls,
            i,
            &log_lines,
            session,
            &render_span_option,
            &mut render_span_session,
        )?;
    }

    let mut funcs_by_name: HashMap<InternedString, Vec<usize>> = HashMap::new();

    for (i, entry) in session.log.iter().enumerate() {
        if let LogEntry::SolveFuncStart { func, .. } = entry {
            match funcs_by_name.entry(func.name) {
                Entry::Occupied(mut e) => {
                    e.get_mut().push(i);
                },
                Entry::Vacant(e) => {
                    e.insert(vec![i]);
                },
            }
        }
    }

    let mut funcs_by_name: Vec<(String, Vec<usize>)> = funcs_by_name.into_iter().map(
        |(name, index)| (name.unintern_or_default(&session.intermediate_dir), index)
    ).collect();
    funcs_by_name.sort_by_key(|(name, _)| name.to_string());

    let mut funcs_by_name_str = vec![];
    funcs_by_name_str.push(String::from("<ul>"));

    for (name, funcs) in funcs_by_name.iter() {
        funcs_by_name_str.push(format!("<li>{name}<ul>"));

        for (i, j) in funcs.iter().enumerate() {
            funcs_by_name_str.push(format!(r#"<li><a href="../{:02}/{j}.html">{i}</a></li>"#, j % 100));
        }

        funcs_by_name_str.push(String::from("</ul></li>"));
    }

    funcs_by_name_str.push(String::from("</ul>"));
    let funcs_by_name_str = funcs_by_name_str.concat();

    let s = format!(r#"
<!DOCTYPE html>
<html>

<head>
<style>
{STYLE}
</style>
</head>

<body>

<a href="../00/0.html">Home</a>
<a href="../indexes/func.html">Func</a>

<h1>Funcs by name</h1>

{funcs_by_name_str}
</body>

</html>
"#);
    let inter_dir = join3(
        &session.intermediate_dir,
        "irs",
        &join3(
            "intermir",
            "log",
            "indexes",
        )?,
    )?;

    if !exists(&inter_dir) {
        create_dir_all(&inter_dir)?;
    }

    let path = join(&inter_dir, &format!("func.html"))?;
    write_string(&path, &s, WriteMode::CreateOrTruncate)?;

    Ok(())
}

fn escape_html(s: &str) -> String {
    s
        .replace("&", "&amp;")
        .replace(">", "&gt;")
        .replace("<", "&lt;")
}
