use sodigy_file::File;
use sodigy_string::InternedString;

mod cmp;
mod derive;
mod endec;
mod render;

pub use derive::SpanDeriveKind;
pub use render::{
    Color,
    ColorOption,
    RenderSpanOption,
    RenderSpanSession,
    RenderableSpan,
    render_spans,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Span {
    // Defspan of lib.
    // Virtually, it's name_span of `mod lib;`.
    Lib,

    // Defspan of stb.
    // Virtually, it's name_span of `mod std;`.
    Std,

    // When a span has something to do with this file, but we cannot tell the exact location.
    // e.g. if there's an error reading the file, the error has this span.
    File(File),

    // When lexer generates a token, the token's span is always `Span::Range`.
    // In the later passes, the compiler might generate tokens. For example, `a && b` is
    // desugared to `if a { b } else { False }`. In this case, the new tokens (`if`, `else`
    // and the curly braces) have `Span::Derived`.
    Range {
        file: File,

        // start..end
        start: usize,
        end: usize,
    },
    Derived {
        kind: SpanDeriveKind,
        file: File,
        start: usize,
        end: usize,
    },

    Eof(File),
    Prelude(InternedString),

    // The compiler might create poly-generic functions while compilation (e.g. associated functions).
    // There are 3 kinds of spans to track:
    //    1. name_span, for the poly-solver
    //    2. generic param for the func params, for the type-checker and generic-solver
    //    3. generic param for the return types, for the type-checker and generic-solver
    Poly {
        name: InternedString,
        kind: PolySpanKind,
    },

    None,
}

impl Span {
    pub fn range(file: File, start: usize, end: usize) -> Self {
        Span::Range { file, start, end }
    }

    pub fn single(file: File, offset: usize) -> Self {
        Span::Range { file, start: offset, end: offset + 1 }
    }

    pub fn eof(file: File) -> Self {
        Span::Eof(file)
    }

    pub fn file(file: File) -> Self {
        Span::File(file)
    }

    #[must_use = "method returns a new span and does not mutate the original span"]
    pub fn merge(&self, other: Span) -> Self {
        match (self, other) {
            (
                Span::Range { file: file1, start: start1, end: end1 } | Span::Derived { file: file1, start: start1, end: end1, .. },
                Span::Range { file: file2, start: start2, end: end2 } | Span::Derived { file: file2, start: start2, end: end2, .. },
            ) if *file1 == file2 => {
                let (file, start, end) = (*file1, (*start1).min(start2), (*end1).max(end2));

                match (self, other) {
                    (Span::Range { .. }, Span::Range { .. }) => Span::Range { file, start, end },
                    (
                        Span::Range { .. } | Span::Derived { kind: SpanDeriveKind::Trivial, .. },
                        Span::Range { .. } | Span::Derived { kind: SpanDeriveKind::Trivial, .. },
                    ) => Span::Derived { kind: SpanDeriveKind::Trivial, file, start, end },
                    (Span::Derived { kind: kind1, .. }, Span::Derived { kind: kind2, .. }) if *kind1 == kind2 => Span::Derived {
                        kind: *kind1,
                        file,
                        start,
                        end,
                    },

                    // I want to preserve as much information as possible, but there are so many cases!!
                    _ => panic!("TODO: {self:?} ++ {other:?}"),
                }
            },
            (Span::None, s) => s,
            (s, Span::None) => *s,
            _ => panic!("TODO: {self:?}, {other:?}"),
        }
    }

    pub fn begin(&self) -> Self {
        match self {
            Span::Range { file, start, .. } => Span::Range {
                file: *file,
                start: *start,
                end: *start + 1,
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
            Span::Derived { kind, file, end, .. } => Span::Derived {
                kind: *kind,
                file: *file,
                start: (*end).max(1) - 1,
                end: *end,
            },
            Span::Lib | Span::Std | Span::None => Span::None,
            Span::Prelude(_) | Span::Poly { .. } => unreachable!(),
        }
    }

    pub fn get_file(&self) -> Option<File> {
        match self {
            Span::File(file) |
            Span::Eof(file) |
            Span::Range { file, .. } |
            Span::Derived { file, .. } => Some(*file),
            Span::Lib |
            Span::Std |
            Span::Prelude(_) |
            Span::Poly { .. } |
            Span::None => None,
        }
    }

    pub fn offset(&mut self, offset: usize) {
        match self {
            Span::Range { start, end, .. } => {
                *start += offset;
                *end += offset;
            },
            _ => {},
        }
    }

    /// An error takes `Vec<RenderableSpan>` as an input,
    /// but we're too lazy to instantiate one.
    pub fn simple_error(&self) -> Vec<RenderableSpan> {
        vec![RenderableSpan {
            span: *self,
            auxiliary: false,
            note: None,
        }]
    }

    pub fn simple_error_with_note(&self, note: &str) -> Vec<RenderableSpan> {
        vec![RenderableSpan {
            span: *self,
            auxiliary: false,
            note: Some(note.to_string()),
        }]
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PolySpanKind {
    Name,
    Param(usize),
    Return,
}
