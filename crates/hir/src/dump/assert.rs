use super::dump_expr;
use crate::{Assert, Session};
use sodigy_endec::IndentedLines;

pub fn dump_assert(assert: &Assert, lines: &mut IndentedLines, session: &Session) {
    lines.break_line();

    lines.push(&format!("// keyword_span: {:?}", assert.keyword_span));
    lines.break_line();

    if let Some(name) = assert.name {
        lines.push(&format!("#[name({:?})]", name.unintern_or_default(&session.intermediate_dir)));
        lines.break_line();
    }

    if let Some(note) = &assert.note {
        lines.push("#[note(");
        dump_expr(note, lines, session);
        lines.push(")]");
        lines.break_line();
    }

    if assert.always {
        lines.push("#[always]");
        lines.break_line();
    }

    lines.push("assert ");
    dump_expr(&assert.value, lines, session);
    lines.push(";");
    lines.break_line();
}
