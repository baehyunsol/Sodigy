use super::{dump_expr, dump_type};
use crate::{Let, Type};
use sodigy_endec::IndentedLines;
use sodigy_session::SodigySession;
use sodigy_span::Span;
use std::collections::HashMap;

// TODO: respect `dump_expr`'s `single_line` option
pub fn dump_let<S: SodigySession>(
    r#let: &Let,
    lines: &mut IndentedLines,
    types: &HashMap<Span, Type>,
    session: &S,
    with_newline: bool,
) {
    if with_newline {
        lines.break_line();
    }

    lines.push(&format!("// name_span: {:?}", r#let.name_span));
    lines.break_line();

    if let Some(type_annot_span) = &r#let.type_annot_span {
        lines.push(&format!("// type_annot_span: {type_annot_span:?}"));
        lines.break_line();
    }

    else {
        lines.push("// There's no type annotation. The type is infered.");
        lines.break_line();
    }

    lines.push(&format!("let {}", r#let.name.unintern_or_default(session.intermediate_dir())));
    lines.push(": ");

    if let Some(r#type) = types.get(&r#let.name_span) {
        dump_type(r#type, lines, session);
    }

    else {
        lines.push("_");
    }

    lines.push(" = ");
    dump_expr(&r#let.value, lines, types, session, 0, false);
    lines.push(";");
    lines.break_line();
}
