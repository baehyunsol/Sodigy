use super::{FuncCall, PreviewMap, Value, create_preview_map, escape_html, to_html};
use sodigy_error::dump_errors;
use sodigy_fs_api::{
    FileError,
    WriteMode,
    create_dir_all,
    exists,
    join,
    join3,
    write_string,
};
use sodigy_inter_mir::{LogEntry, Session as InterMirSession, SolvePolyResult};
use sodigy_mir::{Session as MirSession, Type, dump_field_to_string};
use sodigy_parse::merge_field_spans;
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

pub fn dump_inter_mir_log(session: &InterMirSession, mir_session: &MirSession) -> Result<(), FileError> {
    fn to_func_call(log: &[LogEntry], mut index: usize, session: &InterMirSession, mir_session: &MirSession) -> (FuncCall, usize) {
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

                ("type_solve_loop", e.id())
            },
            LogEntry::SolveSupertypeStart { id, lhs, rhs, lhs_span, rhs_span, context } => {
                input.push(Value {
                    name: String::from("lhs"),
                    short: escape_html(&session.render_type(lhs)),
                    long: Some(String::from_utf8(prettify(format!("{lhs:?}").into_bytes())).unwrap()),
                });
                input.push(Value {
                    name: String::from("rhs"),
                    short: escape_html(&session.render_type(rhs)),
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
            LogEntry::SolvePathStart { id, path, dotfish, prev_infered } => {
                input.push(Value {
                    name: String::from("id"),
                    short: path.id.unintern_or_default(&session.intermediate_dir),
                    long: Some(String::from_utf8(prettify(format!("{path:?}").into_bytes())).unwrap()),
                });

                match dotfish {
                    Some(dotfish) => {
                        input.push(Value {
                            name: String::from("dotfish"),
                            short: String::from("(...)"),
                            long: Some(String::from_utf8(prettify(format!("{dotfish:?}").into_bytes())).unwrap()),
                        });
                    },
                    None => {
                        input.push(Value {
                            name: String::from("dotfish"),
                            short: String::from("N/A"),
                            long: None,
                        });
                    },
                }

                match prev_infered {
                    Some(r#type) => {
                        input.push(Value {
                            name: String::from("prev_infered"),
                            short: escape_html(&session.render_type(r#type)),
                            long: Some(String::from_utf8(prettify(format!("{type:?}").into_bytes())).unwrap()),
                        });
                    },
                    None => {
                        input.push(Value {
                            name: String::from("prev_infered"),
                            short: String::from("N/A"),
                            long: None,
                        });
                    },
                }

                spans.push(RenderableSpan {
                    span: path.def_span.clone(),
                    auxiliary: false,
                    note: Some(String::from("def-span")),
                });
                spans.push(RenderableSpan {
                    span: path.span.clone(),
                    auxiliary: false,
                    note: Some(String::from("span")),
                });

                ("solve_path", *id)
            },
            LogEntry::SolvePatternStart { id, pattern } => {
                input.push(Value {
                    name: String::from("pattern"),
                    short: String::from("(...)"),  // TODO: dump_pattern?
                    long: Some(String::from_utf8(prettify(format!("{pattern:?}").into_bytes())).unwrap()),
                });

                spans.push(RenderableSpan {
                    span: pattern.error_span_wide(),
                    auxiliary: false,
                    note: Some(String::from("pattern")),
                });

                ("solve_pattern", *id)
            },
            LogEntry::GetTypeOfFieldStart { id, r#type, field } => {
                input.push(Value {
                    name: String::from("type"),
                    short: escape_html(&session.render_type(r#type)),
                    long: Some(String::from_utf8(prettify(format!("{type:?}").into_bytes())).unwrap()),
                });

                input.push(Value {
                    name: String::from("field"),
                    short: dump_field_to_string(field, mir_session),
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
                    short: escape_html(&session.render_type(r#type)),
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
            LogEntry::CheckAllTypesInferedStart { id } => ("check_all_types_infered", *id),
            _ => unreachable!(),
        };

        let mut children = vec![];
        index += 1;

        while log[index].id() != log_id {
            let (child, new_index) = to_func_call(log, index, session, mir_session);
            index = new_index;
            children.push(child);
        }

        let mut output = vec![];

        let (has_error, last_errors) = match &log[index] {
            LogEntry::TypeSolveLoopEnd(_) => (false, vec![]),
            LogEntry::SolveSupertypeEnd { solved_type, has_error, last_errors, .. } => {
                output.push(Value::from_optional_type("solved_type", solved_type, session));
                (*has_error, last_errors.clone())
            },
            LogEntry::SolveFuncEnd { annotated_type, infered_type, has_error, last_errors, .. } => {
                output.push(Value {
                    name: String::from("annotated_type"),
                    short: escape_html(&session.render_type(annotated_type)),
                    long: Some(String::from_utf8(prettify(format!("{annotated_type:?}").into_bytes())).unwrap()),
                });
                output.push(Value::from_optional_type("infered_type", infered_type, session));
                (*has_error, last_errors.clone())
            },
            LogEntry::SolveLetEnd { annotated_type, infered_type, has_error, last_errors, .. } => {
                output.push(Value {
                    name: String::from("annotated_type"),
                    short: escape_html(&session.render_type(annotated_type)),
                    long: Some(String::from_utf8(prettify(format!("{annotated_type:?}").into_bytes())).unwrap()),
                });
                output.push(Value::from_optional_type("infered_type", infered_type, session));
                (*has_error, last_errors.clone())
            },
            LogEntry::SolveAssertEnd { has_error, last_errors, .. } => (*has_error, last_errors.clone()),
            LogEntry::SolveExprEnd { infered_type, type_vars, has_error, last_errors, .. } |
            LogEntry::SolvePathEnd { infered_type, type_vars, has_error, last_errors, .. } |
            LogEntry::SolvePatternEnd { infered_type, type_vars, has_error, last_errors, .. } => {
                output.push(Value::from_optional_type("infered_type", infered_type, session));

                if !type_vars.is_empty() {
                    for (i, (type_var, r#type)) in type_vars.iter().enumerate() {
                        match r#type {
                            Some(r#type) => {
                                output.push(Value {
                                    name: format!("type-var-{i}"),
                                    short: escape_html(&session.render_type(r#type)),
                                    long: Some(format!(
                                        "var: {}\n\n------\n\ntype: {}",
                                        String::from_utf8(prettify(format!("{type_var:?}").into_bytes())).unwrap(),
                                        String::from_utf8(prettify(format!("{type:?}").into_bytes())).unwrap(),
                                    )),
                                });
                            },
                            None => {
                                output.push(Value {
                                    name: format!("type-var-{i}"),
                                    short: String::from("N/A"),
                                    long: Some(format!(
                                        "var: {}",
                                        String::from_utf8(prettify(format!("{type_var:?}").into_bytes())).unwrap(),
                                    )),
                                });
                            },
                        }

                        match type_var {
                            Type::Var { def_span, .. } => {
                                spans.push(RenderableSpan {
                                    span: def_span.clone(),
                                    auxiliary: true,
                                    note: Some(format!("type-var-{i}-def")),
                                });
                            },
                            Type::GenericArg { call, generic } => {
                                spans.push(RenderableSpan {
                                    span: call.clone(),
                                    auxiliary: true,
                                    note: Some(format!("type-var-{i}-call")),
                                });
                                spans.push(RenderableSpan {
                                    span: generic.clone(),
                                    auxiliary: true,
                                    note: Some(format!("type-var-{i}-generic")),
                                });
                            },
                            _ => {},
                        }
                    }
                }

                (*has_error, last_errors.clone())
            },
            LogEntry::GetTypeOfFieldEnd { associated_func, infered_type, has_error, last_errors, .. } => {
                // TODO: dump associated_func
                output.push(Value::from_optional_type("infered_type", infered_type, session));
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
            LogEntry::InitPolySolverEnd { solver, state_machine, has_error, last_errors, .. } => {
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

                match state_machine {
                    Some(s) => {
                        output.push(Value {
                            name: String::from("state_machine"),
                            short: String::from("(...)"),
                            long: Some(s.to_string()),
                        });
                    },
                    None => {
                        output.push(Value {
                            name: String::from("state_machine"),
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
                        SolvePolyResult::OneCandidate(_, _) => String::from("one-candidate"),
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
                    SolvePolyResult::OneCandidate(s, _) => {
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
            LogEntry::CheckAllTypesInferedEnd { has_error, last_errors, .. } => (*has_error, last_errors.clone()),
            _ => unreachable!(),
        };
        let last_errors = last_errors.into_iter().map(|(type_error, error)| (Some(type_error), error)).collect();

        let has_inner_error = has_error || children.iter().any(|c| c.has_inner_error);

        (
            FuncCall {
                call_index,
                name: name.to_string(),
                input,
                children,
                output,
                spans,
                has_error,
                last_errors,
                has_inner_error,
            },
            index + 1,
        )
    }

    let mut index = 0;
    let mut calls = vec![];

    while session.log.get(index).is_some() {
        let (call, new_index) = to_func_call(&session.log, index, session, mir_session);
        index = new_index;
        calls.push(call);
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

        for (i, _) in call.children.iter().enumerate() {
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
                                &session.intermediate_dir,
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

    fn render_map_and_save(calls: &[FuncCall], intermediate_dir: &str) -> Result<(), FileError> {
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
            intermediate_dir,
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

    let render_span_option = RenderSpanOption {
        max_height: 20,
        max_width: 160,
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

    render_map_and_save(&calls, &session.intermediate_dir)?;
    Ok(())
}
