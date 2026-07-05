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
use sodigy_inter_mir::{LogEntry, Session as InterMirSession, SolvePolyResult, TypeError};
use sodigy_parse::merge_field_spans;
use sodigy_post_mir::MatchDump;
use sodigy_prettify::prettify;
use sodigy_span::{
    Color,
    ColorOption,
    RenderSpanOption,
    RenderSpanSession,
    RenderableSpan,
    render_spans,
};
use std::collections::HashMap;

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

.preview-map {
    display: inline-block;
    padding: 12px;
    background-color: rgb(0, 0, 0);
    border: 4px solid rgb(255, 255, 255);
    border-radius: 8px;
    overflow: auto;
    width: 300px;
    height: 500px;
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
    background-color: rgb(32, 32, 32);
    padding: 20px;
    border: 4px solid rgb(255, 255, 255);
    border-radius: 8px;
    overflow: auto;
    z-index: 2;
}

.error-marker {
    padding: 4px;
    border-radius: 12px;
    background-color: rgb(255, 0, 0);
}

li {
    margin-top: 12px;
    margin-bottom: 12px;
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

.red {
    color: rgb(192, 32, 32);
}

.green {
    color: rgb(32, 192, 32);
}

.blue {
    color: rgb(32, 32, 192);
}

.yellow {
    color: rgb(192, 192, 32);
}
";

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

pub fn log_inter_mir(session: &InterMirSession) -> Result<(), FileError> {
    struct FuncCall {
        call_index: usize,
        name: String,
        input: Vec<Value>,
        children: Vec<FuncCall>,
        output: Vec<Value>,
        spans: Vec<RenderableSpan>,
        has_error: bool,

        // self.has_error || self.children.any(|c| c.has_inner_error)
        has_inner_error: bool,
        last_errors: Vec<TypeError>,
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

    // It's supposed to return `(FuncCall, usize)`, but some are not implemented yet, so it returns `Option<FuncCall>`.
    fn to_func_call(log: &[LogEntry], mut index: usize, session: &InterMirSession) -> (Option<FuncCall>, usize) {
        let mut spans = vec![];
        let mut input = vec![];

        let call_index = index;
        let (name, log_id) = match &log[index] {
            e @ LogEntry::TypeSolveLoopStart(i) => {
                input.push(Value {
                    name: String::from("i"),
                    short: i.to_string(),
                    long: None,
                });

                ("type_solve_loop", e.id().unwrap())
            },
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
            LogEntry::InitPolySolverStart { id, poly_def_span, poly } => {
                input.push(Value {
                    name: String::from("poly_def_span"),
                    short: String::from("(...)"),
                    long: Some(String::from_utf8(prettify(format!("{poly_def_span:?}").into_bytes())).unwrap()),
                });
                input.push(Value {
                    name: String::from("poly"),
                    short: String::from("(...)"),
                    long: Some(String::from_utf8(prettify(format!("{poly:?}").into_bytes())).unwrap()),
                });

                for (i, r#impl) in poly.impls.iter().enumerate() {
                    spans.push(RenderableSpan {
                        span: r#impl.clone(),
                        auxiliary: true,
                        note: Some(format!("impl-{i}")),
                    });
                }

                spans.push(RenderableSpan {
                    span: poly_def_span.clone(),
                    auxiliary: false,
                    note: Some(String::from("poly_def_span")),
                });

                ("init_poly_solver", *id)
            },
            LogEntry::InitPolySolversStart { id } => ("init_poly_solvers", *id),
            LogEntry::TrySolvePolyStart { id, generic_call, poly, solver } => {
                input.push(Value {
                    name: String::from("generic_call"),
                    short: String::from("(...)"),
                    long: Some(String::from_utf8(prettify(format!("{generic_call:?}").into_bytes())).unwrap()),
                });
                input.push(Value {
                    name: String::from("poly"),
                    short: poly.as_ref().map(|p| p.name.unintern_or_default(&session.intermediate_dir)).unwrap_or(String::from("(...)")),
                    long: Some(String::from_utf8(prettify(format!("{poly:?}").into_bytes())).unwrap()),
                });
                input.push(Value {
                    name: String::from("solver"),
                    short: String::from("(...)"),
                    long: Some(String::from_utf8(prettify(format!("{solver:?}").into_bytes())).unwrap()),
                });

                spans.push(RenderableSpan {
                    span: generic_call.call.clone(),
                    auxiliary: true,
                    note: Some(String::from("generic_call.call")),
                });
                spans.push(RenderableSpan {
                    span: generic_call.def.clone(),
                    auxiliary: true,
                    note: Some(String::from("generic_call.def")),
                });

                if let Some(variant) = &generic_call.variant {
                    spans.push(RenderableSpan {
                        span: variant.clone(),
                        auxiliary: true,
                        note: Some(String::from("generic_call.variant")),
                    });
                }

                for (span, r#type) in generic_call.generics.iter() {
                    spans.push(RenderableSpan {
                        span: span.clone(),
                        auxiliary: true,
                        note: Some(session.render_type(r#type)),
                    });
                }

                if let Some(poly) = poly {
                    spans.push(RenderableSpan {
                        span: poly.name_span.clone(),
                        auxiliary: true,
                        note: Some(String::from("poly.name_span")),
                    });
                }

                ("try_solve_poly", *id)
            },
            LogEntry::MonomorphizeFuncStart { id, func, monomorphization } => {
                input.push(Value {
                    name: String::from("func"),
                    short: func.name.unintern_or_default(&session.intermediate_dir),
                    long: Some(String::from_utf8(prettify(format!("{func:?}").into_bytes())).unwrap()),
                });
                input.push(Value {
                    name: String::from("monomorphization"),
                    short: String::from("(...)"),
                    long: Some(String::from_utf8(prettify(format!("{monomorphization:?}").into_bytes())).unwrap()),
                });

                spans.push(RenderableSpan {
                    span: func.name_span.clone(),
                    auxiliary: true,
                    note: Some(String::from("func")),
                });
                spans.push(RenderableSpan {
                    span: monomorphization.def_span.clone(),
                    auxiliary: true,
                    note: Some(String::from("monomorphization.def_span")),
                });
                spans.push(RenderableSpan {
                    span: monomorphization.call_span.clone(),
                    auxiliary: true,
                    note: Some(String::from("monomorphization.call_span")),
                });

                for (span, r#type) in monomorphization.generics.iter() {
                    spans.push(RenderableSpan {
                        span: span.clone(),
                        auxiliary: true,
                        note: Some(session.render_type(r#type)),
                    });
                }

                ("monomorphize_func", *id)
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
            LogEntry::InitPolySolverEnd { solver, has_error, last_errors, .. } => {
                match solver {
                    Some(s) => {
                        output.push(Value {
                            name: String::from("solver"),
                            short: String::from("(...)"),
                            long: Some(String::from_utf8(prettify(format!("{s:?}").into_bytes())).unwrap()),
                        });
                    },
                    None => {
                        output.push(Value {
                            name: String::from("solver"),
                            short: String::from("N/A"),
                            long: None,
                        });
                    },
                }

                (*has_error, last_errors.clone())
            },
            LogEntry::InitPolySolversEnd { has_error, last_errors, .. } => (*has_error, last_errors.clone()),
            LogEntry::TrySolvePolyEnd { result, .. } => {
                output.push(Value {
                    name: String::from("result"),
                    short: match result {
                        SolvePolyResult::NotPoly => String::from("not-poly"),
                        SolvePolyResult::DefaultImpl(_) => String::from("default-impl"),
                        SolvePolyResult::NoCandidates => String::from("no-candidates"),
                        SolvePolyResult::OneCandidate(_) => String::from("one-candidate"),
                        SolvePolyResult::MultiCandidates(cs) => format!("multi-candidates ({})", cs.len()),
                    },
                    long: Some(String::from_utf8(prettify(format!("{result:?}").into_bytes())).unwrap()),
                });

                match result {
                    SolvePolyResult::DefaultImpl(s) => {
                        spans.push(RenderableSpan {
                            span: s.clone(),
                            auxiliary: true,
                            note: Some(String::from("default-impl")),
                        });
                    },
                    SolvePolyResult::OneCandidate(s) => {
                        spans.push(RenderableSpan {
                            span: s.clone(),
                            auxiliary: true,
                            note: Some(String::from("candidate")),
                        });
                    },
                    SolvePolyResult::MultiCandidates(cs) => {
                        for (i, s) in cs.iter().enumerate() {
                            spans.push(RenderableSpan {
                                span: s.clone(),
                                auxiliary: true,
                                note: Some(format!("candidate-{i}")),
                            });
                        }
                    },
                    SolvePolyResult::NotPoly | SolvePolyResult::NoCandidates => {},
                }

                (false, vec![])
            },
            LogEntry::MonomorphizeFuncEnd { result, .. } => {
                output.push(Value {
                    name: String::from("result"),
                    short: String::from("(...)"),
                    long: Some(String::from_utf8(prettify(format!("{result:?}").into_bytes())).unwrap()),
                });

                (false, vec![])
            },
            _ => todo!(),
        };

        let has_inner_error = has_error || children.iter().any(|c| c.has_inner_error);

        (
            Some(FuncCall {
                call_index,
                name: name.to_string(),
                input,
                children,
                output,
                spans,
                has_error,
                has_inner_error,
                last_errors,
            }),
            index + 1,
        )
    }

    let mut index = 0;
    let mut calls = vec![];

    while session.log.get(index).is_some() {
        let (call, new_index) = to_func_call(&session.log, index, session);
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
        preview_map: &HashMap<usize, PreviewMap>,
        session: &InterMirSession,
        render_span_option: &RenderSpanOption,
        render_span_session: &mut RenderSpanSession,
    ) -> Result<(), FileError> {
        let call = &calls[index];
        let call_index = call.call_index;

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

        for (i, child) in call.children.iter().enumerate() {
            render_page_and_save(
                Some(call_index),
                &call.children,
                i,
                preview_map,
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
        let title = call.title();

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
</ul>
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
        write_string(&path, &to_html(&call_index.to_string(), &body), WriteMode::CreateOrTruncate)?;
        Ok(())
    }

    fn render_map_and_save(calls: &[FuncCall], session: &InterMirSession) -> Result<(), FileError> {
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

        write_string(
            &join(&inter_dir, "map.html")?,
            &to_html("map", &render_map(calls, false, 3)),
            WriteMode::CreateOrTruncate,
        )?;

        write_string(
            &join(&inter_dir, "error.html")?,
            &to_html("error", &render_map(calls, true, 10)),
            WriteMode::CreateOrTruncate,
        )?;

        Ok(())
    }

    let render_span_option = RenderSpanOption {
        max_height: 16,
        max_width: 120,
        context: 8,
        render_source: true,
        color: Some(ColorOption {
            primary: Color::Yellow,
            auxiliary: Color::Yellow,
            info: Color::Green,
        }),
        group_delim: None,
    };
    let mut render_span_session = RenderSpanSession::new(&session.intermediate_dir);
    let preview_map = create_preview_map(&calls);

    for (i, _) in calls.iter().enumerate() {
        render_page_and_save(
            None,
            &calls,
            i,
            &preview_map,
            session,
            &render_span_option,
            &mut render_span_session,
        )?;
    }

    render_map_and_save(&calls, session)?;
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
