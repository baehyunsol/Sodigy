use sodigy_file::File;
use sodigy_string::InternedString;
use sodigy_utils::{dump_hex, hash};
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SpanHash(pub u128);

impl SpanHash {
    pub fn hex(&self, l: usize) -> String {
        dump_hex(self.0, l)
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct SpanId(pub u128);

impl fmt::Debug for SpanId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let file = File(((self.0 >> 64) & 0xffff_ffff) as u32);
        let offset = (self.0 >> 32) & 0xffff_ffff;
        let length = self.0 & 0xffff_ffff;

        write!(
            fmt,
            "{{ file: {file:?}, offset: {offset}, length: {length}, id: {} }}",
            self.0,
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

    // Let's say there's `let x = { ... }; let y = x();`.
    // When we try to solve the type of `y`, it'll get `Type::Blocked(call_span_of_x)`.
    // To move further, we introduce an intermediate type:
    // `Type::Func { return: Type::Var { def_span: IntermediateTypeVar(call_span_of_x), .. }, .. }`.
    IntermediateTypeVar(Box<Span>),

    Std,  // def_span of `std/lib.sdg`
    Lib,  // def_span of `lib.sdg`
    None,
}

impl Span {
    pub fn range(file: File, offset: u32, length: u32) -> Self {
        Span::Range(SpanId(
            ((file.0 as u128) << 64) |
            ((offset as u128) << 32) |
            length as u128
        ))
    }

    pub fn single(file: File, offset: u32) -> Self {
        Span::Range(SpanId(((file.0 as u128) << 64) | ((offset as u128) << 32) | 1))
    }

    #[must_use = "method returns a new span and does not mutate the original span"]
    pub fn merge(&self, other: &Span) -> Self {
        match (self, other) {
            (Span::None, _) => other.clone(),
            (_, Span::None) => self.clone(),
            (Span::Range(_), Span::Range(_)) => {
                let (file1, (offset1, length1)) = (self.file().unwrap(), self.get_offset_and_length().unwrap());
                let (file2, (offset2, length2)) = (other.file().unwrap(), other.get_offset_and_length().unwrap());

                if file1 != file2 {
                    panic!("ICE: {self:?}.merge({other:?})")
                } else {
                    let offset = offset1.min(offset2);
                    let end = (offset1 + length1).max(offset2 + length2);
                    let length = end - offset;
                    Span::range(file1, offset, length)
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
                let (offset, _) = self.get_offset_and_length().unwrap();
                Span::range(self.file().unwrap(), offset, 1)
            },
            Span::Monomorphize { id, span } => Span::Monomorphize {
                id: *id,
                span: Box::new(span.start()),
            },
            Span::Derived { kind, span } => Span::Derived {
                kind: *kind,
                span: Box::new(span.start()),
            },
            Span::Prelude(_) |
            Span::Poly { .. } |
            Span::IntermediateTypeVar(_) |
            Span::Std |
            Span::Lib => self.clone(),
            Span::None => Span::None,
        }
    }

    pub fn end(&self) -> Self {
        match self {
            Span::Range(_) => {
                let (offset, length) = self.get_offset_and_length().unwrap();
                Span::range(self.file().unwrap(), (offset + length).max(1) - 1, 1)
            },
            Span::Monomorphize { id, span } => Span::Monomorphize {
                id: *id,
                span: Box::new(span.end()),
            },
            Span::Derived { kind, span } => Span::Derived {
                kind: *kind,
                span: Box::new(span.end()),
            },
            Span::Prelude(_) |
            Span::Poly { .. } |
            Span::IntermediateTypeVar(_) |
            Span::Std |
            Span::Lib => self.clone(),
            Span::None => Span::None,
        }
    }

    pub fn file(&self) -> Option<File> {
        match self {
            Span::Range(SpanId(r)) => Some(File(((r >> 64) & 0xffff_ffff) as u32)),
            Span::Monomorphize { span, .. } |
            Span::Derived { span, .. } => span.file(),
            Span::None |
            Span::Prelude(_) |
            Span::Poly { .. } |
            Span::IntermediateTypeVar(_) => None,

            // TODO: maybe there's a way to represent this...
            Span::Std | Span::Lib => None,
        }
    }

    pub fn offset(&mut self, offset: u32) {
        match self {
            Span::Range(SpanId(n)) => {
                *n += (offset as u128) << 32;
            },
            Span::Monomorphize { span, .. } |
            Span::Derived { span, .. } => {
                span.offset(offset);
            },
            Span::Prelude(_) |
            Span::Poly { .. } |
            Span::IntermediateTypeVar(_) |
            Span::Std |
            Span::Lib |
            Span::None => {},
        }
    }

    pub fn get_offset_and_length(&self) -> Option<(u32, u32)> {
        match self {
            Span::Range(SpanId(n)) => Some((
                ((*n >> 32) & 0xffff_ffff) as u32,
                (*n & 0xffff_ffff) as u32,
            )),
            Span::Monomorphize { span, .. } |
            Span::Derived { span, .. } => span.get_offset_and_length(),
            Span::Prelude(_) |
            Span::Poly { .. } |
            Span::IntermediateTypeVar(_) |
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

    pub fn id_equals(&self, id: SpanId) -> bool {
        self.id() == Some(id)
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

    pub fn hash(&self) -> SpanHash {
        use sodigy_endec::Endec;
        SpanHash(hash(&self.encode()))
    }

    pub fn or(&self, other: &Span) -> Span {
        match (self, other) {
            (Span::None, _) => other.clone(),
            _ => self.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PolySpanKind {
    Name,
    Param(usize),
    Return,
}
