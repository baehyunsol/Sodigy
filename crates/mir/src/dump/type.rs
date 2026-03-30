use crate::{Session, Type};
use sodigy_endec::IndentedLines;
use sodigy_hir::FuncPurity;
use sodigy_session::SodigySession;
use sodigy_span::{PolySpanKind, Span, SpanId};
use sodigy_string::InternedString;
use std::collections::HashMap;

pub fn dump_type<S: SodigySession>(r#type: &Type, lines: &mut IndentedLines, session: &S) {
    lines.push(&render_type(
        r#type,
        true,  // verbose
        session.lang_items().unwrap_or(&HashMap::new()),
        session.intermediate_dir(),
        session.span_string_map().unwrap_or(&HashMap::new()),
    ));
}

pub fn render_type(
    r#type: &Type,
    verbose: bool,
    lang_items: &HashMap<String, Span>,
    intermediate_dir: &str,

    // inter-mir will initialize this map
    span_string_map: &HashMap<SpanId, InternedString>,
) -> String {
    match r#type {
        Type::Data { constructor_def_span, args, .. } => {
            if let Some(args) = args {
                let args = args.iter().map(
                    |arg| render_type(arg, verbose, lang_items, intermediate_dir, span_string_map)
                ).collect::<Vec<_>>().join(", ");

                if let Some(list_def_span) = lang_items.get("type.List") && list_def_span == constructor_def_span {
                    format!("[{args}]")
                }

                else if let Some(tuple_def_span) = lang_items.get("type.Tuple") && tuple_def_span == constructor_def_span {
                    format!("({args})")
                }

                else {
                    format!("{}<{args}>", if verbose {
                        span_to_string_or_verbose(constructor_def_span, intermediate_dir, span_string_map)
                    } else {
                        span_to_string(constructor_def_span, intermediate_dir, span_string_map).unwrap_or_else(|| String::from("???"))
                    })
                }
            }

            else {
                if verbose {
                    span_to_string_or_verbose(constructor_def_span, intermediate_dir, span_string_map)
                } else {
                    span_to_string(constructor_def_span, intermediate_dir, span_string_map).unwrap_or_else(|| String::from("???"))
                }
            }
        },
        Type::Func { params, r#return, purity, .. } => format!(
            "{}({}) -> {}",
            match purity {
                FuncPurity::Pure => "PureFn",
                FuncPurity::Impure => "ImpureFn",
                FuncPurity::Both => "Fn",
            },
            params.iter().map(
                |param| render_type(param, verbose, lang_items, intermediate_dir, span_string_map)
            ).collect::<Vec<_>>().join(", "),
            render_type(r#return.as_ref(), verbose, lang_items, intermediate_dir, span_string_map),
        ),
        Type::GenericParam { def_span, .. } => if verbose {
            span_to_string_or_verbose(def_span, intermediate_dir, span_string_map)
        } else {
            span_to_string(def_span, intermediate_dir, span_string_map).unwrap_or_else(|| String::from("???"))
        },
        Type::Var { .. } |
        Type::GenericArg { .. } |
        Type::Blocked { .. } => String::from("_"),
        Type::Never { .. } => String::from("!"),
    }
}

pub fn span_to_string(
    span: &Span,
    intermediate_dir: &str,

    // inter-mir will initialize this map
    span_string_map: &HashMap<SpanId, InternedString>,
) -> Option<String> {
    match span {
        Span::Range(r) => match span_string_map.get(r) {
            Some(s) => Some(s.unintern_or_default(intermediate_dir)),
            _ => None,
        },
        Span::Monomorphize { span, .. } | Span::Derived { span, .. } => span_to_string(span, intermediate_dir, span_string_map),
        Span::Prelude(p) => Some(p.unintern_or_default(intermediate_dir)),
        Span::Poly { name, kind } => {
            let name = name.unintern_or_default(intermediate_dir);

            match kind {
                PolySpanKind::Name => Some(name),
                PolySpanKind::Param(i) => Some(format!("T{i}")),
                PolySpanKind::Return => Some(String::from("V")),
            }
        },
        Span::None => None,
        _ => todo!(),
    }
}

pub fn span_to_string_or_verbose(
    span: &Span,
    intermediate_dir: &str,
    span_string_map: &HashMap<SpanId, InternedString>,
) -> String {
    span_to_string(span, intermediate_dir, span_string_map).unwrap_or_else(|| format!("{span:?}"))
}
