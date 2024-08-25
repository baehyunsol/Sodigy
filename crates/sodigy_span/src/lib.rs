#![deny(unused_imports)]

use sodigy_files::{DUMMY_FILE_HASH, FileHash, global_file_session};
use std::collections::hash_map;
use std::hash::Hasher;

mod endec;
mod fmt;
mod render;

pub use render::{ColorScheme, render_spans};

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct SpanPoint {
    file: FileHash,
    index: usize,
}

impl SpanPoint {
    /// Even though it's a dummy, it takes an argument: dummy index.
    /// That's for debugging purpose: when you encounter a dummy span while testing the compiler,
    /// you might wanna know who instantiated this dummy span. `dummy_index` will help you in those cases.
    pub fn dummy(dummy_index: usize) -> Self {
        SpanPoint {
            file: DUMMY_FILE_HASH,
            index: dummy_index,
        }
    }

    pub fn is_dummy(&self) -> bool {
        self.file == DUMMY_FILE_HASH
    }

    /// Read the comments in `Self::dummy()`
    pub fn get_dummy_index(&self) -> Option<usize> {
        if self.is_dummy() {
            Some(self.index)
        }

        else {
            None
        }
    }

    pub fn at_file(file: FileHash, index: usize) -> Self {
        SpanPoint { file, index }
    }

    pub fn extend(self, end: SpanPoint) -> SpanRange {
        debug_assert_eq!(self.file, end.file);

        SpanRange {
            file: self.file,
            start: self.index,
            end: end.index,
            is_real: true,  // SpanPoint is always real
        }
    }

    #[must_use = "method returns a new span and does not mutate the original value"]
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
            is_real: true,  // SpanPoint is always real
        }
    }
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SpanRange {
    pub file: FileHash,
    start: usize,  // inclusive
    end: usize,    // exclusive

    // sometimes the compiler has to create code
    // those code use spans of nearby code so that the error message gets nicer
    pub is_real: bool,
}

impl SpanRange {
    /// make sure not to call `render_spans` on dummy spans
    pub fn dummy() -> SpanRange {
        SpanRange {
            file: DUMMY_FILE_HASH,
            start: 0,
            end: 0,
            is_real: false,
        }
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

    pub fn first_char(&self) -> SpanRange {
        SpanRange {
            file: self.file,
            start: self.start,
            end: self.start + 1,
            is_real: self.is_real,
        }
    }

    // don't use span.end.into_range() -> span.end is exclusive!
    pub fn last_char(&self) -> SpanRange {
        SpanRange {
            file: self.file,
            start: self.end - 1,
            end: self.end,
            is_real: self.is_real,
        }
    }

    #[must_use = "method returns a new span and does not mutate the original value"]
    pub fn merge(&self, other: SpanRange) -> Self {
        debug_assert!(self.end <= other.start);
        debug_assert!(self.file == other.file);

        SpanRange {
            file: self.file,
            start: self.start,
            end: other.end,
            is_real: self.is_real && other.is_real,
        }
    }

    pub fn into_fake(&self) -> Self {
        SpanRange {
            is_real: false,
            ..*self
        }
    }

    pub fn hash128(&self) -> u128 {
        // self.file is already a hash value
        ((self.file as u128) << 64) | {
            let mut hasher = hash_map::DefaultHasher::new();
            hasher.write(&(self.start as u64).to_be_bytes());
            hasher.write(&(self.end as u64).to_be_bytes());
            hasher.write(&[self.is_real as u8]);

            hasher.finish() as u128
        }
    }

    // reads the actual file and convert the span to the original string
    /// EXPENSIVE
    pub fn to_utf8(&self) -> Vec<u8> {
        unsafe {
            let g = global_file_session();

            match g.get_file_content(self.file) {
                Ok(buffer) => buffer[self.start..self.end].to_vec(),
                Err(e) => panic!("{e:?}"),
            }
        }
    }
}
