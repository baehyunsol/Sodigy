use super::{SpanPoint, SpanRange};
use sodigy_files::global_file_session;
use std::fmt;

impl fmt::Debug for SpanPoint {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            fmt,
            "Span({}, {})",
            unsafe { global_file_session().render_file_hash(self.file) },
            self.index,
        )
    }
}

impl fmt::Debug for SpanRange {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            fmt,
            "Span({}, {}, {})",
            unsafe { global_file_session().render_file_hash(self.file) },
            self.start,
            self.end,
        )
    }
}
