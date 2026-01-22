use super::{dump_expr, dump_type};
use crate::{Func, Session, Type};
use sodigy_endec::IndentedLines;

pub fn dump_func(func: &Func, lines: &mut IndentedLines, session: &Session) {
    lines.break_line();

    lines.push(&format!("// name_span: {:?}", func.name_span));
    lines.break_line();

    if func.built_in {
        lines.push("#[built_in]");
        lines.break_line();
    }

    if !func.is_pure {
        lines.push("impure ");
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
        lines.push(": ");

        if let Some(type_annot) = session.types.get(&param.name_span) {
            dump_type(type_annot, lines, session);
        }

        else {
            lines.push("_");
        }

        if let Some(default_value) = param.default_value {
            lines.push(&format!(" = {}", default_value.id.unintern_or_default(&session.intermediate_dir)));
        }

        lines.push(",");
    }

    lines.push(") -> ");

    if let Some(Type::Func { r#return, .. }) = session.types.get(&func.name_span) {
        dump_type(r#return, lines, session);
    }

    else {
        lines.push("_");
    }

    if !func.built_in {
        lines.push(" = ");
        dump_expr(&func.value, lines, session);
    }

    lines.push(";");
    lines.break_line();
}
