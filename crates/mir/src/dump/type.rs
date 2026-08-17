use crate::Type;
use sodigy_endec::IndentedLines;
use sodigy_error::FuncEffect;
use sodigy_session::SodigySession;
use sodigy_span::{PolySpanKind, Span, SpanId};
use sodigy_string::{InternedString, unintern_string};
use std::collections::HashMap;

pub fn dump_type<S: SodigySession>(r#type: &Type, lines: &mut IndentedLines, session: &S) {
    lines.push(&render_type(
        r#type,
        true,  // verbose
        session.lang_items().unwrap_or(&HashMap::new()),
        session.intermediate_dir(),
        session.span_string_map().unwrap_or(&HashMap::new()),
    ).unwrap_or(String::from("????")));
}

pub fn render_type(
    r#type: &Type,
    verbose: bool,
    lang_items: &HashMap<String, Span>,
    intermediate_dir: &str,

    // inter-mir will initialize this map
    span_string_map: &HashMap<SpanId, InternedString>,
) -> Option<String> {
    match r#type {
        Type::Data { constructor_def_span, args, .. } => {
            if let Some(args) = args {
                let mut args_rendered = Vec::with_capacity(args.len());

                for arg in args.iter() {
                    args_rendered.push(render_type(arg, verbose, lang_items, intermediate_dir, span_string_map)?);
                }

                let args = args_rendered.join(", ");

                if let Some(list_def_span) = lang_items.get("type.List") && list_def_span.id_equals(*constructor_def_span) {
                    Some(format!("[{args}]"))
                }

                else if let Some(tuple_def_span) = lang_items.get("type.Tuple") && tuple_def_span.id_equals(*constructor_def_span) {
                    Some(format!("({args})"))
                }

                else if verbose {
                    Some(format!("{}<{args}>", span_to_string_or_verbose(&Span::Range(*constructor_def_span), intermediate_dir, span_string_map)))
                }

                else {
                    Some(format!("{}<{args}>", span_to_string(&Span::Range(*constructor_def_span), intermediate_dir, span_string_map)?))
                }
            }

            else if verbose {
                Some(span_to_string_or_verbose(&Span::Range(*constructor_def_span), intermediate_dir, span_string_map))
            }

            else {
                span_to_string(&Span::Range(*constructor_def_span), intermediate_dir, span_string_map)
            }
        },
        Type::Func { params, r#return, effect, .. } => {
            let effect = match effect {
                FuncEffect::Fn => "Fn",
                FuncEffect::Proc => "Proc",
                FuncEffect::NdetFn => "NdetFn",
                FuncEffect::NdetProc => "NdetProc",
                FuncEffect::Callable => "Callable",
                FuncEffect::Var(_) => "EffectVar",
            };
            let mut params_rendered = Vec::with_capacity(params.len());

            for param in params.iter() {
                params_rendered.push(render_type(param, verbose, lang_items, intermediate_dir, span_string_map)?);
            }

            let params = params_rendered.join(", ");
            let r#return = render_type(r#return, verbose, lang_items, intermediate_dir, span_string_map)?;
            Some(format!("{effect}({params}) -> {return}"))
        },
        Type::GenericParam { def_span, .. } => if verbose {
            Some(span_to_string_or_verbose(def_span, intermediate_dir, span_string_map))
        } else {
            Some(span_to_string(def_span, intermediate_dir, span_string_map)?)
        },
        Type::Var { .. } |
        Type::GenericArg { .. } |
        Type::Blocked { .. } => Some(String::from("_")),
        Type::Never { .. } => Some(String::from("!")),
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
            Some(s) => unintern_ident(*s, intermediate_dir),
            _ => None,
        },
        Span::Monomorphize { span, .. } | Span::Derived { span, .. } => span_to_string(span, intermediate_dir, span_string_map),
        Span::Prelude(p) => unintern_ident(*p, intermediate_dir),
        Span::Poly { name, kind } => {
            let name = unintern_ident(*name, intermediate_dir)?;

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

fn unintern_ident(id: InternedString, intermediate_dir: &str) -> Option<String> {
    match unintern_string(id, intermediate_dir) {
        Err(_) | Ok(None) => None,

        // If the identifier is an invalid utf-8, that's an ICE.
        Ok(Some(b)) => Some(String::from_utf8(b).unwrap()),
    }
}
