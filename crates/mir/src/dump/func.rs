use super::{dump_expr, dump_type};
use crate::{Func, Session, Type};
use sodigy_endec::IndentedLines;
use sodigy_session::SodigySession;
use sodigy_span::Span;
use std::collections::HashMap;

// TODO: respect `dump_expr`'s `single_line` option
pub fn dump_func<S: SodigySession>(
    func: &Func,
    lines: &mut IndentedLines,
    types: &HashMap<Span, Type>,
    session: &S,
) {
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

    lines.push(&format!("fn {}", func.name.unintern_or_default(session.intermediate_dir())));

    if !func.generics.is_empty() {
        lines.push("<");

        for generic in func.generics.iter() {
            lines.push(&generic.name.unintern_or_default(session.intermediate_dir()));
            lines.push(",");
        }

        lines.push(">");
    }

    lines.push("(");

    for param in func.params.iter() {
        lines.push(&param.name.unintern_or_default(session.intermediate_dir()));
        lines.push(": ");

        if let Some(type_annot) = types.get(&param.name_span) {
            dump_type(type_annot, lines, session);
        }

        else {
            lines.push("_");
        }

        if let Some(default_value) = &param.default_value {
            lines.push(&format!(" = {}", default_value.id.unintern_or_default(session.intermediate_dir())));
        }

        lines.push(",");
    }

    lines.push(") -> ");

    if let Some(Type::Func { r#return, .. }) = types.get(&func.name_span) {
        dump_type(r#return.as_ref(), lines, session);
    }

    else {
        lines.push("_");
    }

    if !func.built_in {
        lines.push(" = ");
        dump_expr(&func.value, lines, types, session, 0, false);
    }

    lines.push(";");
    lines.break_line();
}
