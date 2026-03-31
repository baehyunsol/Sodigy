use super::dump_expr;
use crate::{Assert, Type};
use sodigy_endec::IndentedLines;
use sodigy_session::SodigySession;
use sodigy_span::Span;
use std::collections::HashMap;

// TODO: respect `dump_expr`'s `single_line` option
pub fn dump_assert<S: SodigySession>(
    assert: &Assert,
    lines: &mut IndentedLines,
    types: &HashMap<Span, Type>,
    session: &S,
) {
    lines.break_line();

    lines.push(&format!("// keyword_span: {:?}", assert.keyword_span));
    lines.break_line();

    if let Some(name) = assert.name {
        lines.push(&format!("#[name({:?})]", name.unintern_or_default(session.intermediate_dir())));
        lines.break_line();
    }

    if let Some(note) = &assert.note {
        lines.push("#[note(");
        dump_expr(note, lines, types, session, 0, false);
        lines.push(")]");
        lines.break_line();
    }

    if assert.always {
        lines.push("#[always]");
        lines.break_line();
    }

    lines.push("assert ");
    dump_expr(&assert.value, lines, types, session, 0, false);
    lines.push(";");
    lines.break_line();
}
