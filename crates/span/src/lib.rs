use sodigy_file::File;
use sodigy_string::{InternedString, hash};

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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Span {
    Range(u128),
    Monomorphize {
        // TODO: can we save more space if we use 64 bit id?
        id: u128,
        span: Box<Span>,
    },
    Derived {
        kind: SpanDeriveKind,
        span: Box<Span>,
    },
}

impl Span {
    pub fn dummy() -> Self {
        Span::Range(0)
    }

    pub fn is_dummy(&self) -> bool {
        &Span::Range(0) == self
    }

    pub fn range(file: File, start: u32, end: u32) -> Self {
        Span::Range(
            ((file.0 as u128) << 64) |
            ((start as u128) << 32) |
            end as u128
        )
    }

    pub fn single(file: File, offset: usize) -> Self {
        Span::Range(
            ((file.0 as u128) << 64) |
            ((offset as u128) << 32) |
            (end as u128 + 1)
        )
    }

    #[must_use = "method returns a new span and does not mutate the original span"]
    pub fn merge(&self, other: Span) -> Self {
        todo!()
    }

    pub fn start(&self) -> Self {
        match self {
            _ if self.is_dummy() => Span::dummy(),
            Span::Range(_) => {
                let (start, _) = span_offset(self);
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
        }
    }

    pub fn end(&self) -> Self {
        match self {
            _ if self.is_dummy() => Span::dummy(),
            Span::Range(_) => {
                let (_, end) = span_offset(self);
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
        }
    }

    pub fn file(&self) -> Option<File> {
        match self {
            _ if self.is_dummy() => None,
            Span::Range(_) => todo!(),
            Span::Monomorphize { span, .. } |
            Span::Derived { span, .. } => span.file(),
        }
    }

    pub fn offset(&mut self, offset: u32) {
        match self {
            Span::Range(n) => {
                *n += offset as u128;
                *n += (offset as u128) << 32;
            },
            Span::Monomorphize { span, .. } |
            Span::Derived { span, .. } => {
                span.offset(offset);
            },
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

fn span_offset(span: &Span) -> (u32, u32) {
    match span {
        Span::Range(n) => (
            ((*n >> 32) & 0xffff_ffff) as u32,
            (*n & 0xffff_ffff) as u32,
        ),
        Span::Monomorphize { span, .. } |
        Span::Derived { span, .. } => span_offset(&span),
    }
}
