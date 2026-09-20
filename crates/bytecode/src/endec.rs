use crate::{GlobalLabel, LocalLabel, Memory, SSA};
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
            Memory::SSA(i) => {
                buffer.push(0);
                i.encode_impl(buffer);
            },
            Memory::Heap { ptr, offset } => {
                buffer.push(1);
                ptr.encode_impl(buffer);
                offset.encode_impl(buffer);
            },
            Memory::List { ptr, offset } => {
                buffer.push(2);
                ptr.encode_impl(buffer);
                offset.encode_impl(buffer);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (i, cursor) = SSA::decode_impl(buffer, cursor + 1)?;
                Ok((Memory::SSA(i), cursor))
            },
            Some(1) => {
                let (ptr, cursor) = SSA::decode_impl(buffer, cursor + 1)?;
                let (offset, cursor) = u32::decode_impl(buffer, cursor)?;
                Ok((Memory::Heap { ptr, offset }, cursor))
            },
            Some(2) => {
                let (ptr, cursor) = SSA::decode_impl(buffer, cursor + 1)?;
                let (offset, cursor) = u32::decode_impl(buffer, cursor)?;
                Ok((Memory::List { ptr, offset }, cursor))
            },
            Some(n @ 3..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for LocalLabel {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.index().encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (label, cursor) = u32::decode_impl(buffer, cursor)?;
        Ok((LocalLabel::new(label), cursor))
    }
}

impl Endec for GlobalLabel {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.span().encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (label, cursor) = SpanHash::decode_impl(buffer, cursor)?;
        Ok((GlobalLabel::new(label), cursor))
    }
}
