use crate::{Session, Type};
use sodigy_endec::IndentedLines;

pub fn dump_type(r#type: &Type, lines: &mut IndentedLines, session: &Session) {
    lines.push(&session.render_type(r#type));
}
