use sodigy_files::{DUMMY_FILE_HASH, FileHash, global_file_session};
use sodigy_test::{sodigy_assert, sodigy_assert_eq};

mod fmt;
mod render;

pub use render::render_spans;

#[derive(Clone, Copy)]
pub struct SpanPoint {
    file: FileHash,
    index: usize,
}

impl SpanPoint {
    pub fn dummy() -> Self {
        SpanPoint {
            file: DUMMY_FILE_HASH,
            index: 0,
        }
    }

    pub fn is_dummy(&self) -> bool {
        self.file == DUMMY_FILE_HASH
    }

    pub fn at_file(file: FileHash, index: usize) -> Self {
        SpanPoint { file, index }
    }

    pub fn extend(self, end: SpanPoint) -> SpanRange {
        sodigy_assert_eq!(self.file, end.file);

        SpanRange {
            file: self.file,
            start: self.index,
            end: end.index,
        }
    }

    #[must_use = "method returns a new number and does not mutate the original value"]
    pub fn offset(&self, offset: i32) -> Self {
        SpanPoint {
            file: self.file,
            index: (self.index as i32 + offset) as usize,
        }
    }

    pub fn into_range(&self) -> SpanRange {
        SpanRange {
            file: self.file,
            start: self.index,
            end: self.index + 1,
        }
    }
}

#[derive(Clone, Copy)]
pub struct SpanRange {
    pub file: FileHash,
    start: usize,  // inclusive
    end: usize,    // exclusive
}

impl SpanRange {
    pub fn dummy() -> Self {
        SpanRange {
            file: DUMMY_FILE_HASH,
            start: 0,
            end: 0,
        }
    }

    pub fn is_dummy(&self) -> bool {
        self.file == DUMMY_FILE_HASH
    }

    pub fn start(&self) -> SpanPoint {
        SpanPoint {
            file: self.file,
            index: self.start,
        }
    }

    pub fn end(&self) -> SpanPoint {
        SpanPoint {
            file: self.file,
            index: self.end,
        }
    }

    // don't use span.end.into_range() -> span.end is exclusive!
    pub fn last_char(&self) -> SpanRange {
        SpanRange {
            file: self.file,
            start: self.end - 1,
            end: self.end,
        }
    }

    #[must_use = "method returns a new number and does not mutate the original value"]
    pub fn merge(&self, other: SpanRange) -> Self {
        sodigy_assert!(self.end <= other.start);

        SpanRange {
            file: self.file,
            start: self.start,
            end: other.end,
        }
    }

    // reads the actual file and convert the span to the original string
    /// EXPENSIVE
    pub fn to_utf8(&self) -> Vec<u8> {
        if self.is_dummy() {
            return b"This is a dummy span".to_vec();
        }

        unsafe {
            let g = global_file_session();

            match g.get_tmp_file(self.file) {
                Some(buf) => {
                    buf[self.start..self.end].to_vec()
                },
                _ => match g.get_file_content(self.file) {
                    Ok(buf) => {
                        buf[self.start..self.end].to_vec()
                    },
                    Err(e) => panic!("{e:?}"),
                },
            }
        }
    }
}