use super::{dump_expr, dump_type, dump_visibility};
use crate::{Func, Session};
use sodigy_endec::IndentedLines;

pub fn dump_func(func: &Func, lines: &mut IndentedLines, session: &Session) {
    lines.break_line();

    lines.push(&format!("// name_span: {:?}", func.name_span));
    lines.break_line();
    lines.push(&format!("// origin: {:?}", func.origin));
    lines.break_line();
    lines.push(&format!("// foreign_names: {:?}", func.foreign_names));
    lines.break_line();

    if func.built_in {
        lines.push("#[built_in]");
        lines.break_line();
    }

    let curr_len = lines.total_chars();
    dump_visibility(&func.visibility, lines, session);

    if !func.is_pure {
        if curr_len < lines.total_chars() {
            lines.push(" ");
        }

        lines.push("impure");
    }

    if curr_len < lines.total_chars() {
        lines.push(" ");
    }

    lines.push(&format!("fn {}", func.name.unintern_or_default(&session.intermediate_dir)));

    if !func.generics.is_empty() {
        lines.push("<");

        for generic in func.generics.iter() {
            lines.push(&generic.name.unintern_or_default(&session.intermediate_dir));
            lines.push(",");
        }

        lines.push(">");
    }

    lines.push("(");

    for param in func.params.iter() {
        lines.push(&param.name.unintern_or_default(&session.intermediate_dir));

        if let Some(type_annot) = &param.type_annot {
            lines.push(": ");
            dump_type(type_annot, lines, session);
        }

        if let Some(default_value) = param.default_value {
            lines.push(&format!(" = {}", default_value.id.unintern_or_default(&session.intermediate_dir)));
        }

        lines.push(",");
    }

    lines.push(")");

    if let Some(type_annot) = &func.type_annot {
        lines.push(" -> ");
        dump_type(type_annot, lines, session);
    }

    lines.push(" = ");
    dump_expr(&func.value, lines, session);
    lines.push(";");
    lines.break_line();
}
