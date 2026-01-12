use super::{dump_expr, dump_type, dump_visibility};
use crate::{Let, Session};
use sodigy_endec::IndentedLines;

pub fn dump_let(r#let: &Let, lines: &mut IndentedLines, session: &Session, with_newline: bool) {
    if with_newline {
        lines.break_line();
    }

    lines.push(&format!("// name_span: {:?}", r#let.name_span));
    lines.break_line();
    lines.push(&format!("// origin: {:?}", r#let.origin));
    lines.break_line();
    lines.push(&format!("// foreign_names: {:?}", r#let.foreign_names));
    lines.break_line();

    let curr_len = lines.total_chars();
    dump_visibility(&r#let.visibility, lines, session);

    // visibility token has dumped something, so we need a whitespace.
    if curr_len < lines.total_chars() {
        lines.push(" ");
    }

    lines.push(&format!("let {}", r#let.name.unintern_or_default(&session.intermediate_dir)));

    if let Some(type_annot) = &r#let.type_annot {
        lines.push(": ");
        dump_type(type_annot, lines, session);
    }

    lines.push(" = ");
    dump_expr(&r#let.value, lines, session);
    lines.push(";");
    lines.break_line();
}
