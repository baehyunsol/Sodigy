use crate::{CompileStage, Error};
use sodigy_endec::{DumpSession, Endec};
use sodigy_fs_api::{
    FileError,
    WriteMode,
    create_dir,
    exists,
    join4,
    parent,
    read_bytes,
    write_bytes,
};
use sodigy_inter_mir as inter_mir;
use sodigy_span::{RenderSpanOption, RenderSpanSession, render_spans};
use sodigy_prettify::prettify;

/// The compiler stores irs (or result) in various places.
/// 1. It can store the output to user-given path.
/// 2. If it has to interpret the bytecodes, it just stores them in memory and directly executes them.
/// 3. In a complicated compilation process, it stores irs in the intermediate_dir.
#[derive(Clone, Debug)]
pub enum StoreIrAt {
    File(String),
    IntermediateDir,
}

#[derive(Clone, Debug)]
pub struct EmitIrOption {
    pub stage: CompileStage,
    pub store: StoreIrAt,
    pub human_readable: bool,
}

pub fn emit_irs_if_has_to<T: Endec + DumpSession>(
    session: &T,
    emit_ir_options: &[EmitIrOption],
    finished_stage: CompileStage,
    content_hash: Option<u128>,
    intermediate_dir: &str,
) -> Result<(), Error> {
    let (mut binary, mut human_readable) = (false, false);
    let stores = emit_ir_options.iter().filter(
        |option| option.stage == finished_stage
    ).map(
        |option| {
            if option.human_readable {
                human_readable = true;
            } else {
                binary = true;
            }

            (option.store.clone(), option.human_readable)
        }
    ).collect::<Vec<_>>();
    let binary = if binary {
        Some(session.encode())
    } else {
        None
    };
    let human_readable = if human_readable {
        Some(session.dump_session())
    } else {
        None
    };

    for (store, human_readable_) in stores.iter() {
        let content = if *human_readable_ {
            human_readable.as_ref().unwrap()
        } else {
            binary.as_ref().unwrap()
        };
        let ext = if *human_readable_ { ".rs" } else { "" };

        match store {
            StoreIrAt::File(s) => {
                write_bytes(&s, content, WriteMode::Atomic)?;
            },
            StoreIrAt::IntermediateDir => {
                let path = join4(
                    intermediate_dir,
                    "irs",
                    &format!("{finished_stage:?}").to_lowercase(),
                    &format!(
                        "{}{ext}",
                        if let Some(content_hash) = content_hash {
                            format!("{content_hash:x}")
                        } else {
                            String::from("total")
                        },
                    ),
                )?;
                let parent = parent(&path)?;

                if !exists(&parent) {
                    create_dir(&parent)?;
                }

                write_bytes(
                    &path,
                    content,
                    WriteMode::Atomic,
                )?;
            },
        }
    }

    Ok(())
}

pub fn get_cached_ir(
    intermediate_dir: &str,
    stage: CompileStage,
    content_hash: Option<u128>,
) -> Result<Option<Vec<u8>>, FileError> {
    let path = join4(
        intermediate_dir,
        "irs",
        &format!("{stage:?}").to_lowercase(),
        // There's no `ext` because it's always `!human_readable`
        &if let Some(content_hash) = content_hash {
            format!("{content_hash:x}")
        } else {
            String::from("total")
        },
    )?;

    if exists(&path) {
        Ok(Some(read_bytes(&path)?))
    }

    else {
        Ok(None)
    }
}

pub fn store_inter_mir_log(session: &inter_mir::Session) -> Result<(), FileError> {
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
            LogEntry::SolveSupertype { lhs, rhs, lhs_span, rhs_span, context } => {
                buffer.push(b"\n---\n".to_vec());
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
                let info = session.render_monomorphization_info(&monomorphization);

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
                            session.span_to_string(*param).unwrap_or(String::from("????")),
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

    write_bytes(
        &save_at,
        &buffer.concat(),
        WriteMode::CreateOrTruncate,
    )
}
