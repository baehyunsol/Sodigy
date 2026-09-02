use super::{
    FuncCall,
    Value,
    create_preview_map,
    render_map_and_save,
    render_page_and_save,
};
use sodigy_endec::Endec;
use sodigy_file::File;
use sodigy_fs_api::{
    FileError,
    FileErrorKind,
    WriteMode,
    create_dir,
    exists,
    join,
    join4,
    read_bytes,
    read_dir,
    write_bytes,
};
use sodigy_inter_hir::LogEntry;
use sodigy_prettify::prettify;
use sodigy_span::{
    Color,
    ColorOption,
    RenderSpanOption,
    RenderSpanSession,
    RenderableSpan,
};
use sodigy_utils::hash;

pub fn store_inter_hir_log(
    module: Option<File>,
    log: Vec<LogEntry>,
    intermediate_dir: &str,
) -> Result<(), FileError> {
    if log.is_empty() {
        return Ok(());
    }

    let store_at = join(intermediate_dir, "inter_hir_logs")?;

    if !exists(&store_at) {
        create_dir(&store_at)?;
    }

    let module_hash = format!("{:032x}", hash(&module.encode()));
    let store_at = join(&store_at, &module_hash)?;
    write_bytes(
        &store_at,
        &(module, log).encode(),
        WriteMode::CreateOrTruncate,
    )
}

pub fn dump_inter_hir_log(intermediate_dir: &str) -> Result<(), FileError> {
    let store_at = join(intermediate_dir, "inter_hir_logs")?;

    if !exists(&store_at) {
        return Ok(());
    }

    let mut logs: Vec<(Option<File>, Vec<LogEntry>)> = vec![];

    for file in read_dir(&store_at, true)? {
        let b = read_bytes(&file)?;
        let log: (Option<File>, Vec<LogEntry>) = Endec::decode(&b).map_err(|_| FileError { kind: FileErrorKind::CannotDecodeFile, given_path: Some(file) })?;
        logs.push(log);
    }

    logs.sort_by_key(|(f, _)| *f);
    let logs: Vec<LogEntry> = logs.into_iter().map(|(_, l)| l).collect::<Vec<_>>().concat();
    dump_inter_hir_log_worker(&logs, intermediate_dir)?;
    Ok(())
}

fn dump_inter_hir_log_worker(log: &[LogEntry], intermediate_dir: &str) -> Result<(), FileError> {
    fn to_func_call(log: &[LogEntry], mut index: usize, intermediate_dir: &str) -> (FuncCall, usize) {
        let mut spans = vec![];
        let mut input = vec![];

        let call_index = index;
        let (name, log_id) = match &log[index] {
            LogEntry::ResolveAliasStart { id } => ("resolve_alias", *id),
            e @ LogEntry::ResolveAliasLoopStart(i) => {
                input.push(Value {
                    name: String::from("i"),
                    short: i.to_string(),
                    long: None,
                });

                ("resolve_alias_loop", e.id())
            },
            LogEntry::ResolveItemStart { id, kind, name, span } => {
                input.push(Value {
                    name: String::from("kind"),
                    short: kind.render().to_string(),
                    long: None,
                });

                match name {
                    Some(name) => {
                        input.push(Value {
                            name: String::from("name"),
                            short: name.unintern_or_default(intermediate_dir),
                            long: None,
                        });
                    },
                    None => {
                        input.push(Value {
                            name: String::from("name"),
                            short: String::from("N/A"),
                            long: None,
                        });
                    },
                }

                spans.push(RenderableSpan {
                    span: span.clone(),
                    auxiliary: false,
                    note: None,
                });

                ("resolve_item", *id)
            },
            LogEntry::ResolveUseStart { id, r#use } => {
                input.push(Value {
                    name: String::from("use"),
                    short: format!("use {} as {}", r#use.path.unintern_or_default(intermediate_dir), r#use.name.unintern_or_default(intermediate_dir)),
                    long: Some(String::from_utf8(prettify(format!("{use:?}").into_bytes())).unwrap()),
                });

                spans.push(RenderableSpan {
                    span: r#use.keyword_span.clone(),
                    auxiliary: true,
                    note: Some(String::from("use.keyword_span (before)")),
                });
                spans.push(RenderableSpan {
                    span: r#use.name_span.clone(),
                    auxiliary: true,
                    note: Some(String::from("use.name_span (before)")),
                });
                spans.push(RenderableSpan {
                    span: r#use.path.id.span.clone(),
                    auxiliary: true,
                    note: Some(String::from("use.path.id.span (before)")),
                });
                spans.push(RenderableSpan {
                    span: r#use.path.id.def_span.clone(),
                    auxiliary: true,
                    note: Some(String::from("use.path.id.def_span (before)")),
                });

                ("resolve_use", *id)
            },
            LogEntry::ResolvePathStart { id, path, type_args } => {
                input.push(Value {
                    name: String::from("path"),
                    short: path.unintern_or_default(intermediate_dir),
                    long: Some(String::from_utf8(prettify(format!("{path:?}").into_bytes())).unwrap()),
                });

                match type_args {
                    Some(type_args) => {
                        input.push(Value {
                            name: String::from("type_args"),

                            // inter_hir_session doesn't implement `.render_type()`
                            // because it doesn't have a span_string_map.
                            short: String::from("[...]"),

                            long: Some(String::from_utf8(prettify(format!("{type_args:?}").into_bytes())).unwrap()),
                        });
                    },
                    None => {
                        input.push(Value {
                            name: String::from("type_args"),
                            short: String::from("N/A"),
                            long: None,
                        });
                    },
                }

                spans.push(RenderableSpan {
                    span: path.id.span.clone(),
                    auxiliary: true,
                    note: Some(String::from("path.id.span (before)")),
                });
                spans.push(RenderableSpan {
                    span: path.id.def_span.clone(),
                    auxiliary: true,
                    note: Some(String::from("path.id.def_span (before)")),
                });

                ("resolve_path", *id)
            },
            LogEntry::ResolveExprStart { id, expr } => {
                input.push(Value {
                    name: String::from("expr"),
                    short: String::from("(...)"),  // TODO: dump_expr
                    long: Some(String::from_utf8(prettify(format!("{expr:?}").into_bytes())).unwrap()),
                });

                spans.push(RenderableSpan {
                    span: expr.error_span_wide(),
                    auxiliary: true,
                    note: Some(String::from("expr")),
                });

                ("resolve_expr", *id)
            },
            LogEntry::ResolvePatternStart { id, pattern } => {
                input.push(Value {
                    name: String::from("pattern"),
                    short: String::from("(...)"),  // TODO: dump_pattern
                    long: Some(String::from_utf8(prettify(format!("{pattern:?}").into_bytes())).unwrap()),
                });

                spans.push(RenderableSpan {
                    span: pattern.error_span_wide(),
                    auxiliary: true,
                    note: Some(String::from("pattern")),
                });

                ("resolve_pattern", *id)
            },
            LogEntry::ResolveTypeStart { id, r#type } => {
                input.push(Value {
                    name: String::from("type"),
                    short: String::from("(...)"),  // TODO: dump_type
                    long: Some(String::from_utf8(prettify(format!("{type:?}").into_bytes())).unwrap()),
                });

                spans.push(RenderableSpan {
                    span: r#type.error_span_wide(),
                    auxiliary: true,
                    note: Some(String::from("type")),
                });

                ("resolve_type", *id)
            },
            _ => unreachable!(),
        };

        let mut children = vec![];
        index += 1;

        while log[index].id() != log_id {
            let (child, new_index) = to_func_call(log, index, intermediate_dir);
            index = new_index;
            children.push(child);
        }

        let mut output = vec![];
        let (has_error, last_errors) = match &log[index] {
            LogEntry::ResolveAliasEnd { has_error, last_errors, .. } |
            LogEntry::ResolveItemEnd { has_error, last_errors, .. } => (*has_error, last_errors.clone()),
            LogEntry::ResolveAliasLoopEnd(_) => (false, vec![]),
            LogEntry::ResolveUseEnd { r#use, has_error, last_errors, .. } => {
                output.push(Value {
                    name: String::from("use"),
                    short: String::from("(...)"),  // TODO: dump
                    long: Some(String::from_utf8(prettify(format!("{use:?}").into_bytes())).unwrap()),
                });

                spans.push(RenderableSpan {
                    span: r#use.keyword_span.clone(),
                    auxiliary: true,
                    note: Some(String::from("use.keyword_span (after)")),
                });
                spans.push(RenderableSpan {
                    span: r#use.name_span.clone(),
                    auxiliary: true,
                    note: Some(String::from("use.name_span (after)")),
                });
                spans.push(RenderableSpan {
                    span: r#use.path.id.span.clone(),
                    auxiliary: true,
                    note: Some(String::from("use.path.id.span (after)")),
                });
                spans.push(RenderableSpan {
                    span: r#use.path.id.def_span.clone(),
                    auxiliary: true,
                    note: Some(String::from("use.path.id.def_span (after)")),
                });

                (*has_error, last_errors.clone())
            },
            LogEntry::ResolvePathEnd { path, has_error, last_errors, .. } => {
                output.push(Value {
                    name: String::from("path"),
                    short: path.unintern_or_default(intermediate_dir),
                    long: Some(String::from_utf8(prettify(format!("{path:?}").into_bytes())).unwrap()),
                });

                spans.push(RenderableSpan {
                    span: path.id.span.clone(),
                    auxiliary: true,
                    note: Some(String::from("path.id.span (after)")),
                });
                spans.push(RenderableSpan {
                    span: path.id.def_span.clone(),
                    auxiliary: true,
                    note: Some(String::from("path.id.def_span (after)")),
                });

                (*has_error, last_errors.clone())
            },
            LogEntry::ResolveExprEnd { expr, has_error, last_errors, .. } => {
                output.push(Value {
                    name: String::from("expr"),
                    short: String::from("(...)"),
                    long: Some(String::from_utf8(prettify(format!("{expr:?}").into_bytes())).unwrap()),
                });

                (*has_error, last_errors.clone())
            },
            LogEntry::ResolvePatternEnd { pattern, has_error, last_errors, .. } => {
                output.push(Value {
                    name: String::from("pattern"),
                    short: String::from("(...)"),
                    long: Some(String::from_utf8(prettify(format!("{pattern:?}").into_bytes())).unwrap()),
                });

                (*has_error, last_errors.clone())
            },
            LogEntry::ResolveTypeEnd { r#type, has_error, last_errors, .. } => {
                output.push(Value {
                    name: String::from("type"),
                    short: String::from("(...)"),
                    long: Some(String::from_utf8(prettify(format!("{type:?}").into_bytes())).unwrap()),
                });

                (*has_error, last_errors.clone())
            },
            _ => unreachable!(),
        };
        let last_errors = last_errors.into_iter().map(|error| (None, error)).collect();
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

    while log.get(index).is_some() {
        let (call, new_index) = to_func_call(log, index, intermediate_dir);
        index = new_index;
        calls.push(call);
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
    let mut render_span_session = RenderSpanSession::new(intermediate_dir);
    let preview_map = create_preview_map(&calls);
    let log_dir = join4(
        intermediate_dir,
        "irs",
        "interhir",
        "log",
    )?;

    for (i, _) in calls.iter().enumerate() {
        render_page_and_save(
            None,
            &calls,
            i,
            &preview_map,
            &log_dir,
            intermediate_dir,
            &render_span_option,
            &mut render_span_session,
        )?;
    }

    render_map_and_save(&calls, &log_dir)?;
    Ok(())
}
