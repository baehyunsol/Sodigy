use sodigy_fs_api::{
    FileError,
    WriteMode,
    create_dir,
    exists,
    join4,
    parent,
    write_bytes,
    write_string,
};
use sodigy_inter_mir as inter_mir;
use sodigy_span::{
    RenderSpanOption,
    RenderSpanSession,
    RenderableSpan,
    render_spans,
};
use sodigy_post_mir::MatchDump;
use sodigy_prettify::prettify;
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

pub fn log_inter_mir(session: &inter_mir::Session) -> Result<(), FileError> {
    use inter_mir::LogEntry;

    if session.log.is_empty() {
        return Ok(());
    }

    let mut buffer = vec![];
    let mut render_span_session = RenderSpanSession::new(&session.intermediate_dir);
    let render_span_option = RenderSpanOption {
        max_width: 128,
        max_height: 12,
        context: 5,
        render_source: true,
        color: None,
        group_delim: None,
    };

    for entry in session.log.iter() {
        match entry {
            LogEntry::TypeSolveLoopStart(i) => {
                buffer.push(format!("\n-------- TypeSolveLoopStart({i}) --------").into_bytes());
            },
            LogEntry::SolveSupertype { lhs, rhs, lhs_span, rhs_span, context } => {
                buffer.push(b"\n-------- SolveSupertype --------\n".to_vec());
                buffer.push(prettify(format!("{entry:?}\n").into_bytes()));
                buffer.push(format!("lhs: {}\n", session.render_type(lhs)).into_bytes());
                buffer.push(format!("rhs: {}\n", session.render_type(rhs)).into_bytes());
                buffer.push(format!("context: {context:?}\n").into_bytes());

                if let Some(lhs_span) = lhs_span {
                    buffer.push(format!("lhs_span:\n{}\n", render_spans(
                        &lhs_span.simple_error(),
                        &render_span_option,
                        &mut render_span_session,
                    )).into_bytes());
                }

                if let Some(rhs_span) = rhs_span {
                    buffer.push(format!("rhs_span:\n{}\n", render_spans(
                        &rhs_span.simple_error(),
                        &render_span_option,
                        &mut render_span_session,
                    )).into_bytes());
                }
            },
            LogEntry::SolveFunc { func, annotated_type, infered_type } => {
                buffer.push(b"\n-------- SolveFunc --------\n".to_vec());
                buffer.push(prettify(format!("{entry:?}\n").into_bytes()));
                buffer.push(format!("func_name: {}\n", func.name.unintern_or_default(&session.intermediate_dir)).into_bytes());
                buffer.push(format!("annotated_type: {}\n", session.render_type(annotated_type)).into_bytes());
                buffer.push(format!("infered_type: {}\n", infered_type.as_ref().map(|t| session.render_type(t)).unwrap_or(String::from("(failed to infer the type)"))).into_bytes());
                buffer.push(format!("func_span:\n{}\n", render_spans(
                    &func.name_span.simple_error(),
                    &render_span_option,
                    &mut render_span_session,
                )).into_bytes());
            },
            LogEntry::SolveLet { r#let, annotated_type, infered_type } => {
                buffer.push(b"\n-------- SolveLet --------\n".to_vec());
                buffer.push(prettify(format!("{entry:?}\n").into_bytes()));
                buffer.push(format!("let_name: {}\n", r#let.name.unintern_or_default(&session.intermediate_dir)).into_bytes());
                buffer.push(format!("annotated_type: {}\n", session.render_type(annotated_type)).into_bytes());
                buffer.push(format!("infered_type: {}\n", infered_type.as_ref().map(|t| session.render_type(t)).unwrap_or(String::from("(failed to infer the type)"))).into_bytes());
                buffer.push(format!("let_span:\n{}\n", render_spans(
                    &r#let.name_span.simple_error(),
                    &render_span_option,
                    &mut render_span_session,
                )).into_bytes());
            },
            LogEntry::Monomorphization(monomorphization) => {
                let info = session.render_monomorphization_info(monomorphization);

                buffer.push(b"\n-------- Monomorphization --------\n".to_vec());
                buffer.push(prettify(format!("{entry:?}\n").into_bytes()));
                buffer.push(format!("id_dec: {}\n", monomorphization.id).into_bytes());
                buffer.push(format!("id_hex: {:x}\n", monomorphization.id).into_bytes());
                buffer.push(format!("def: {}\n", info.info).into_bytes());
                buffer.push(format!("def_span:\n{}\n", render_spans(
                    &monomorphization.def_span.simple_error(),
                    &render_span_option,
                    &mut render_span_session,
                )).into_bytes());
                buffer.push(format!("call_span:\n{}\n", render_spans(
                    &monomorphization.call_span.simple_error(),
                    &render_span_option,
                    &mut render_span_session,
                )).into_bytes());
            },
            LogEntry::InitPolySolver { poly_def_span, solver } => {
                buffer.push(b"\n-------- InitPolySolver --------\n".to_vec());
                // buffer.push(prettify(format!("{entry:?}\n").into_bytes()));  // Too long
                let mut spans = vec![RenderableSpan {
                    span: poly_def_span.clone(),
                    auxiliary: false,
                    note: Some(String::from("This is the #[poly] definition.")),
                }];
                let mut impl_name_map = HashMap::new();

                for impl_span in solver.impls.keys() {
                    let name = format!(
                        "impl-{}-{}",
                        session.span_to_string(impl_span).unwrap_or(String::from("????")),
                        impl_name_map.len(),
                    );
                    spans.push(RenderableSpan {
                        span: impl_span.clone(),
                        auxiliary: true,
                        note: Some(format!("This is `{name}`.")),
                    });
                    impl_name_map.insert(impl_span.clone(), name);
                }

                buffer.push(render_spans(&spans, &render_span_option, &mut render_span_session).into_bytes());
                buffer.push(b"\nstate machine:\n```\n".to_vec());

                if let Some(state_machine) = &solver.state_machine {
                    buffer.push(session.render_state_machine(state_machine, &impl_name_map).into_bytes());
                }

                else {
                    buffer.push(b"It has no state machine.".to_vec());
                }

                buffer.push(b"\n```\n".to_vec());
            },
            LogEntry::TrySolvePoly { generic_call, poly_def, result } => {
                buffer.push(b"\n-------- TrySolvePoly --------\n".to_vec());
                buffer.push(prettify(format!("{entry:?}\n").into_bytes()));
                buffer.push(format!("call_span:\n{}\n", render_spans(
                    &generic_call.call.simple_error(),
                    &render_span_option,
                    &mut render_span_session,
                )).into_bytes());
                buffer.push(format!("def_span:\n{}\n", render_spans(
                    &generic_call.def.simple_error(),
                    &render_span_option,
                    &mut render_span_session,
                )).into_bytes());
                buffer.push(format!(
                    "generics: {{{}}}\n",
                    generic_call.generics.iter().map(
                        |(param, r#type)| format!(
                            "{}: {}",
                            session.span_to_string(param).unwrap_or(String::from("????")),
                            session.render_type(r#type),
                        )
                    ).collect::<Vec<_>>().join(", "),
                ).into_bytes());
            },
            LogEntry::AssociatedFunc { def_span, call_span } => {
                buffer.push(b"\n-------- AssociatedFunc --------\n".to_vec());
                buffer.push(prettify(format!("{entry:?}\n").into_bytes()));
                buffer.push(format!("call_span:\n{}\n", render_spans(
                    &call_span.simple_error(),
                    &render_span_option,
                    &mut render_span_session,
                )).into_bytes());

                // TODO: what's the point of rendering def_span?
                //       it's always `Span::Poly { .. }`, which cannot be rendered...
                buffer.push(format!("def_span:\n{}\n", render_spans(
                    &def_span.simple_error(),
                    &render_span_option,
                    &mut render_span_session,
                )).into_bytes());
            },
            LogEntry::BlockedTypeVar { kind: _, span } => {
                buffer.push(b"\n-------- BlockedTypeVar --------\n".to_vec());
                buffer.push(prettify(format!("{entry:?}\n").into_bytes()));
                buffer.push(format!("span:\n{}\n", render_spans(
                    &span.simple_error(),
                    &render_span_option,
                    &mut render_span_session,
                )).into_bytes());
            },
            LogEntry::TypeError { type_error, general_error } => {
                buffer.push(b"\n-------- TypeError --------\n".to_vec());
                buffer.push(prettify(format!("{entry:?}\n").into_bytes()));
                buffer.push(format!("error span:\n{}\n", render_spans(
                    &general_error.spans,
                    &render_span_option,
                    &mut render_span_session,
                )).into_bytes());
            },
        }
    }

    let save_at = join4(
        &session.intermediate_dir,
        "irs",
        "intermir",
        "log",
    )?;

    if !exists(&parent(&save_at)?) {
        create_dir(&parent(&save_at)?)?;
    }

    if !exists(&save_at) {
        write_bytes(&save_at, b"", WriteMode::AlwaysCreate)?;
    }

    write_bytes(
        &save_at,
        &buffer.concat(),
        WriteMode::AlwaysAppend,
    )
}
