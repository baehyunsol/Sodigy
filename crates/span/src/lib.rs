use sodigy_file::File;
use sodigy_string::{InternedString, hash};
use std::fmt;

mod cmp;
mod derive;
mod endec;
mod render;

#[cfg(test)]
mod tests;

pub use derive::SpanDeriveKind;
pub use render::{
    Color,
    ColorOption,
    MonomorphizationInfo,
    RenderSpanOption,
    RenderSpanSession,
    RenderableSpan,
    render_spans,
};

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct SpanId(pub u128);

impl fmt::Debug for SpanId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let file_id = (self.0 >> 64) & 0xffff_ffff;
        let file_type = if file_id >= 0x8000_0000 { "Std" } else { "File" };
        let file_id = file_id & 0x7fff_ffff;
        let start = (self.0 >> 32) & 0xffff_ffff;
        let end = self.0 & 0xffff_ffff;

        write!(
            fmt,
            "{{ file: {file_type}({file_id}), start: {start}, end: {end} }}",
        )
    }
}

// Span is used everywhere and we have to do our best to keep it small.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Span {
    Range(SpanId),
    Monomorphize {
        id: u64,
        span: Box<Span>,
    },
    Derived {
        kind: SpanDeriveKind,
        span: Box<Span>,
    },
    Prelude(InternedString),
    Poly {
        name: InternedString,
        kind: PolySpanKind,
    },
    Std,  // def_span of `std/lib.sdg`
    Lib,  // def_span of `lib.sdg`
    None,
}

impl Span {
    pub fn range(file: File, start: u32, end: u32) -> Self {
        Span::Range(SpanId(
            ((file.0 as u128) << 64) |
            ((start as u128) << 32) |
            end as u128
        ))
    }

    pub fn single(file: File, offset: u32) -> Self {
        Span::Range(SpanId(
            ((file.0 as u128) << 64) |
            ((offset as u128) << 32) |
            (offset as u128 + 1)
        ))
    }

    #[must_use = "method returns a new span and does not mutate the original span"]
    pub fn merge(&self, other: &Span) -> Self {
        match (self, other) {
            (Span::None, _) => other.clone(),
            (_, Span::None) => self.clone(),
            (Span::Range(_), Span::Range(_)) => {
                let (f1, (s1, e1)) = (self.file().unwrap(), self.get_bounds().unwrap());
                let (f2, (s2, e2)) = (self.file().unwrap(), self.get_bounds().unwrap());

                if f1 != f2 {
                    todo!()
                } else {
                    Span::range(f1, s1.min(s2), e1.max(e2))
                }
            },
            (Span::Monomorphize { id, span }, s) |
            (s, Span::Monomorphize { id, span }) => Span::Monomorphize {
                id: *id,
                span: Box::new(span.merge(s)),
            },
            (Span::Derived { kind, span }, s) |
            (s, Span::Derived { kind, span }) => Span::Derived {
                kind: *kind,
                span: Box::new(span.merge(s)),
            },
            s => panic!("TODO: {s:?}"),
        }
    }

    pub fn start(&self) -> Self {
        match self {
            Span::Range(_) => {
                let (start, _) = self.get_bounds().unwrap();
                Span::range(self.file().unwrap(), start, start + 1)
            },
            Span::Monomorphize { id, span } => Span::Monomorphize {
                id: *id,
                span: Box::new(span.start()),
            },
            Span::Derived { kind, span } => Span::Derived {
                kind: *kind,
                span: Box::new(span.start()),
            },
            Span::Prelude(_) | Span::Poly { .. } | Span::Std | Span::Lib => self.clone(),
            Span::None => Span::None,
        }
    }

    pub fn end(&self) -> Self {
        match self {
            Span::Range(_) => {
                let (_, end) = self.get_bounds().unwrap();
                Span::range(self.file().unwrap(), end.max(1) - 1, end)
            },
            Span::Monomorphize { id, span } => Span::Monomorphize {
                id: *id,
                span: Box::new(span.end()),
            },
            Span::Derived { kind, span } => Span::Derived {
                kind: *kind,
                span: Box::new(span.end()),
            },
            Span::Prelude(_) | Span::Poly { .. } | Span::Std | Span::Lib => self.clone(),
            Span::None => Span::None,
        }
    }

    pub fn file(&self) -> Option<File> {
        match self {
            Span::Range(SpanId(r)) => Some(File(((r >> 64) & 0xffff_ffff) as u32)),
            Span::Monomorphize { span, .. } |
            Span::Derived { span, .. } => span.file(),
            Span::None | Span::Prelude(_) | Span::Poly { .. } => None,
            Span::Std | Span::Lib => todo!(),
        }
    }

    pub fn offset(&mut self, offset: u32) {
        match self {
            Span::Range(SpanId(n)) => {
                *n += offset as u128;
                *n += (offset as u128) << 32;
            },
            Span::Monomorphize { span, .. } |
            Span::Derived { span, .. } => {
                span.offset(offset);
            },
            Span::Prelude(_) |
            Span::Poly { .. } |
            Span::Std |
            Span::Lib |
            Span::None => {},
        }
    }

    pub fn get_bounds(&self) -> Option<(u32, u32)> {
        match self {
            Span::Range(SpanId(n)) => Some((
                ((*n >> 32) & 0xffff_ffff) as u32,
                (*n & 0xffff_ffff) as u32,
            )),
            Span::Monomorphize { span, .. } |
            Span::Derived { span, .. } => span.get_bounds(),
            Span::Prelude(_) |
            Span::Poly { .. } |
            Span::Std |
            Span::Lib |
            Span::None => None,
        }
    }

    pub fn id(&self) -> Option<SpanId> {
        match self {
            Span::Range(r) => Some(*r),
            Span::Monomorphize { span, .. } | Span::Derived { span, .. } => span.id(),
            _ => None,
        }
    }

    /// An error takes `Vec<RenderableSpan>` as an input,
    /// but we're too lazy to instantiate one.
    pub fn simple_error(&self) -> Vec<RenderableSpan> {
        vec![RenderableSpan {
            span: self.clone(),
            auxiliary: false,
            note: None,
        }]
    }

    pub fn simple_error_with_note(&self, note: &str) -> Vec<RenderableSpan> {
        vec![RenderableSpan {
            span: self.clone(),
            auxiliary: false,
            note: Some(note.to_string()),
        }]
    }

    pub fn hash(&self) -> u128 {
        use sodigy_endec::Endec;
        hash(&self.encode())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PolySpanKind {
    Name,
    Param(usize),
    Return,
}
