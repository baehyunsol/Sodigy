use crate::{DebugInfoKind, GlobalLabel, LocalLabel, Memory, SSA};
use sodigy_endec::{DecodeError, Endec};
use sodigy_span::SpanHash;

mod assert;
mod bytecode;
mod expr_hash;
mod func;
mod r#let;
mod object_file;
mod session;
mod value;

impl Endec for SSA {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.0.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (ssa, cursor) = u32::decode_impl(buffer, cursor)?;
        Ok((SSA(ssa), cursor))
    }
}

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
                let (i, cursor) = SSA::decode_impl(buffer, cursor + 1)?;
                Ok((Memory::SSA(i), cursor))
            },
            Some(2) => {
                let (ptr, cursor) = SSA::decode_impl(buffer, cursor + 1)?;
                let (offset, cursor) = u32::decode_impl(buffer, cursor)?;
                Ok((Memory::Heap { ptr, offset }, cursor))
            },
            Some(3) => {
                let (ptr, cursor) = SSA::decode_impl(buffer, cursor + 1)?;
                let (offset, cursor) = u32::decode_impl(buffer, cursor)?;
                Ok((Memory::List { ptr, offset }, cursor))
            },
            Some(4) => {
                let (span, cursor) = SpanHash::decode_impl(buffer, cursor + 1)?;
                Ok((Memory::Global(span), cursor))
            },
            Some(n @ 5..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for LocalLabel {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.0.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (label, cursor) = u32::decode_impl(buffer, cursor)?;
        Ok((LocalLabel(label), cursor))
    }
}

impl Endec for GlobalLabel {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.0.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (label, cursor) = SpanHash::decode_impl(buffer, cursor)?;
        Ok((GlobalLabel(label), cursor))
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
