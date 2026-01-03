use crate::{Session, Visibility};
use sodigy_endec::IndentedLines;

pub fn dump_visibility(visibility: &Visibility, lines: &mut IndentedLines, session: &Session) {
    if let Some(_) = visibility.keyword_span {
        lines.push("pub");
    }
}
