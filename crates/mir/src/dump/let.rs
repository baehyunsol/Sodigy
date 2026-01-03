use super::{dump_expr, dump_type};
use crate::{Let, Session};
use sodigy_endec::IndentedLines;

pub fn dump_let(r#let: &Let, lines: &mut IndentedLines, session: &Session) {
    lines.break_line();

    lines.push(&format!("// name_span: {:?}", r#let.name_span));
    lines.break_line();

    if let Some(type_annot_span) = r#let.type_annot_span {
        lines.push("// type_annot_span: {type_annot_span:?}");
        lines.break_line();
    }

    else {
        lines.push("// There's no type annotation. The type is infered.");
        lines.break_line();
    }

    lines.push(&format!(" let {}", r#let.name.unintern_or_default(&session.intermediate_dir)));
    lines.push(": ");

    if let Some(r#type) = session.types.get(&r#let.name_span) {
        dump_type(r#type, lines, session);
    }

    else {
        lines.push("_");
    }

    lines.push(" = ");
    dump_expr(&r#let.value, lines, session);
    lines.push(";");
    lines.break_line();
}
