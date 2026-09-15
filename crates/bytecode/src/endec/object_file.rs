use crate::{
    BasicBlock,
    Bytecode,
    CodeKind,
    CodeSection,
    ExprHash,
    GlobalLabel,
    LocalLabel,
    Memory,
    ObjectFile,
    SSA,
    Terminator,
    Value,
};
use sodigy_endec::{DecodeError, Endec};
use sodigy_error::FuncEffect;
use sodigy_span::{Span, SpanHash};
use std::collections::HashMap;

impl Endec for ObjectFile {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.data.encode_impl(buffer);
        self.code.encode_impl(buffer);
        self.main_entry.encode_impl(buffer);
        self.asserts.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (data, cursor) = Vec::<(ExprHash, Value)>::decode_impl(buffer, cursor)?;
        let (code, cursor) = Vec::<CodeSection>::decode_impl(buffer, cursor)?;
        let (main_entry, cursor) = Option::<SpanHash>::decode_impl(buffer, cursor)?;
        let (asserts, cursor) = Vec::<SpanHash>::decode_impl(buffer, cursor)?;

        Ok((ObjectFile { data, code, main_entry, asserts }, cursor))
    }
}

impl Endec for CodeSection {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.label.encode_impl(buffer);
        self.span.encode_impl(buffer);
        self.kind.encode_impl(buffer);
        self.name.encode_impl(buffer);
        self.params.encode_impl(buffer);
        self.effect.encode_impl(buffer);
        self.basic_blocks.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (label, cursor) = GlobalLabel::decode_impl(buffer, cursor)?;
        let (span, cursor) = Option::<Span>::decode_impl(buffer, cursor)?;
        let (kind, cursor) = CodeKind::decode_impl(buffer, cursor)?;
        let (name, cursor) = String::decode_impl(buffer, cursor)?;
        let (params, cursor) = Option::<usize>::decode_impl(buffer, cursor)?;
        let (effect, cursor) = FuncEffect::decode_impl(buffer, cursor)?;
        let (basic_blocks, cursor) = HashMap::<LocalLabel, BasicBlock>::decode_impl(buffer, cursor)?;

        Ok((CodeSection { label, span, kind, name, params, effect, basic_blocks }, cursor))
    }
}

impl Endec for CodeKind {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            CodeKind::Func => {
                buffer.push(0);
            },
            CodeKind::Let => {
                buffer.push(1);
            },
            CodeKind::Assert => {
                buffer.push(2);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((CodeKind::Func, cursor + 1)),
            Some(1) => Ok((CodeKind::Let, cursor + 1)),
            Some(2) => Ok((CodeKind::Assert, cursor + 1)),
            Some(n @ 3..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for BasicBlock {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.label.encode_impl(buffer);
        self.code.encode_impl(buffer);
        self.terminator.encode_impl(buffer);
        self.terminator_debug_info.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (label, cursor) = LocalLabel::decode_impl(buffer, cursor)?;
        let (code, cursor) = Vec::<Bytecode>::decode_impl(buffer, cursor)?;
        let (terminator, cursor) = Terminator::decode_impl(buffer, cursor)?;
        let (terminator_debug_info, cursor) = Option::<Box<Span>>::decode_impl(buffer, cursor)?;

        Ok((BasicBlock { label, code, terminator, terminator_debug_info }, cursor))
    }
}

impl Endec for Terminator {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            Terminator::Jump(label) => {
                buffer.push(0);
                label.encode_impl(buffer);
            },
            Terminator::TailCall { func, args } => {
                buffer.push(1);
                func.encode_impl(buffer);
                args.encode_impl(buffer);
            },
            Terminator::TailCallDynamic { func, args } => {
                buffer.push(2);
                func.encode_impl(buffer);
                args.encode_impl(buffer);
            },
            Terminator::JumpIf { value, t, f } => {
                buffer.push(3);
                value.encode_impl(buffer);
                t.encode_impl(buffer);
                f.encode_impl(buffer);
            },
            Terminator::Return(src) => {
                buffer.push(4);
                src.encode_impl(buffer);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (label, cursor) = LocalLabel::decode_impl(buffer, cursor + 1)?;
                Ok((Terminator::Jump(label), cursor))
            },
            Some(1) => {
                let (func, cursor) = GlobalLabel::decode_impl(buffer, cursor + 1)?;
                let (args, cursor) = Vec::<SSA>::decode_impl(buffer, cursor)?;
                Ok((Terminator::TailCall { func, args }, cursor))
            },
            Some(2) => {
                let (func, cursor) = SSA::decode_impl(buffer, cursor + 1)?;
                let (args, cursor) = Vec::<SSA>::decode_impl(buffer, cursor)?;
                Ok((Terminator::TailCallDynamic { func, args }, cursor))
            },
            Some(3) => {
                let (value, cursor) = Memory::decode_impl(buffer, cursor + 1)?;
                let (t, cursor) = LocalLabel::decode_impl(buffer, cursor)?;
                let (f, cursor) = LocalLabel::decode_impl(buffer, cursor)?;
                Ok((Terminator::JumpIf { value, t, f }, cursor))
            },
            Some(4) => {
                let (src, cursor) = SSA::decode_impl(buffer, cursor + 1)?;
                Ok((Terminator::Return(src), cursor))
            },
            Some(n @ 5..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
