use sodigy_file::File;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Span {
    // When a span has something to do with this file, but we cannot tell the exact location.
    // e.g. if there's an error reading the file, the error has this span.
    File(File),
    Range {
        file: File,

        // start..end
        start: usize,
        end: usize,
    },
    Eof(File),
    None,
}

impl Span {
    pub fn range(file: File, start: usize, end: usize) -> Self {
        Span::Range { file, start, end }
    }

    pub fn eof(file: File) -> Self {
        Span::Eof(file)
    }

    pub fn file(file: File) -> Self {
        Span::File(file)
    }

    #[must_use = "method returns a new span and does not mutate the original value"]
    pub fn merge(&self, other: Span) -> Self {
        match (self, other) {
            (
                Span::Range { file: file1, start: start1, end: end1 },
                Span::Range { file: file2, start: start2, end: end2 },
            ) if *file1 == file2 => Span::Range {
                file: *file1,
                start: (*start1).min(start2),
                end: (*end1).max(end2),
            },
            _ => todo!(),
        }
    }

    pub fn end(&self) -> Self {
        match self {
            Span::File(file) | Span::Eof(file) => Span::Eof(*file),
            Span::Range { file, end, .. } => Span::Range {
                file: *file,
                start: (*end).max(1) - 1,
                end: *end,
            },
            Span::None => Span::None,
        }
    }
}
