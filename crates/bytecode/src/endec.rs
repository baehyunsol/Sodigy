use crate::{DebugInfoKind, Label, Memory, Offset};
use sodigy_endec::{DecodeError, Endec};
use sodigy_span::Span;

mod assert;
mod bytecode;
mod executable;
mod func;
mod r#let;
mod session;
mod value;

impl Endec for Memory {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            Memory::Return => {
                buffer.push(0);
            },
            Memory::SSA(i) => {
                buffer.push(1);
                i.encode_impl(buffer);
            },
            Memory::Heap { ptr, offset } => {
                buffer.push(2);
                ptr.encode_impl(buffer);
                offset.encode_impl(buffer);
            },
            Memory::List { ptr, offset } => {
                buffer.push(3);
                ptr.encode_impl(buffer);
                offset.encode_impl(buffer);
            },
            Memory::Global(span) => {
                buffer.push(4);
                span.encode_impl(buffer);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((Memory::Return, cursor + 1)),
            Some(1) => {
                let (i, cursor) = u32::decode_impl(buffer, cursor + 1)?;
                Ok((Memory::SSA(i), cursor))
            },
            Some(2) => {
                let (ptr, cursor) = Box::<Memory>::decode_impl(buffer, cursor + 1)?;
                let (offset, cursor) = Offset::decode_impl(buffer, cursor)?;
                Ok((Memory::Heap { ptr, offset }, cursor))
            },
            Some(3) => {
                let (ptr, cursor) = Box::<Memory>::decode_impl(buffer, cursor + 1)?;
                let (offset, cursor) = Offset::decode_impl(buffer, cursor)?;
                Ok((Memory::List { ptr, offset }, cursor))
            },
            Some(4) => {
                let (span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                Ok((Memory::Global(span), cursor))
            },
            Some(n @ 5..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for Label {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            Label::Local(i) => {
                buffer.push(0);
                i.encode_impl(buffer);
            },
            Label::Global(span) => {
                buffer.push(1);
                span.encode_impl(buffer);
            },
            Label::Flatten(n) => {
                buffer.push(2);
                n.encode_impl(buffer);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (i, cursor) = u32::decode_impl(buffer, cursor + 1)?;
                Ok((Label::Local(i), cursor))
            },
            Some(1) => {
                let (span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                Ok((Label::Global(span), cursor))
            },
            Some(2) => {
                let (n, cursor) = usize::decode_impl(buffer, cursor + 1)?;
                Ok((Label::Flatten(n), cursor))
            },
            Some(n @ 3..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for Offset {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            Offset::Static(n) => {
                buffer.push(0);
                n.encode_impl(buffer);
            },
            Offset::Dynamic(src) => {
                buffer.push(1);
                src.encode_impl(buffer);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (n, cursor) = u32::decode_impl(buffer, cursor + 1)?;
                Ok((Offset::Static(n), cursor))
            },
            Some(1) => {
                let (src, cursor) = Box::<Memory>::decode_impl(buffer, cursor + 1)?;
                Ok((Offset::Dynamic(src), cursor))
            },
            Some(n @ 2..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for DebugInfoKind {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            DebugInfoKind::AssertionKeywordSpan => {
                buffer.push(0);
            },
            DebugInfoKind::AssertionName => {
                buffer.push(1);
            },
            DebugInfoKind::AssertionNoteDecoratorSpan => {
                buffer.push(2);
            },
            DebugInfoKind::AssertionNote => {
                buffer.push(3);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((DebugInfoKind::AssertionKeywordSpan, cursor + 1)),
            Some(1) => Ok((DebugInfoKind::AssertionName, cursor + 1)),
            Some(2) => Ok((DebugInfoKind::AssertionNoteDecoratorSpan, cursor + 1)),
            Some(3) => Ok((DebugInfoKind::AssertionNote, cursor + 1)),
            Some(n @ 4..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
