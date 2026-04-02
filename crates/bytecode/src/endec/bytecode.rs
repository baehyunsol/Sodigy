use crate::{
    Bytecode,
    DebugInfoKind,
    DropType,
    Label,
    Memory,
    Value,
};
use sodigy_endec::{DecodeError, Endec};
use sodigy_mir::Intrinsic;
use sodigy_span::Span;

impl Endec for Bytecode {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            Bytecode::Const { value, dst } => {
                buffer.push(0);
                value.encode_impl(buffer);
                dst.encode_impl(buffer);
            },
            Bytecode::Move { src, dst } => {
                buffer.push(1);
                src.encode_impl(buffer);
                dst.encode_impl(buffer);
            },
            Bytecode::Phi { pair, dst } => {
                buffer.push(2);
                pair.encode_impl(buffer);
                dst.encode_impl(buffer);
            },
            Bytecode::Jump(dst) => {
                buffer.push(3);
                dst.encode_impl(buffer);
            },
            Bytecode::Call { func, args, tail } => {
                buffer.push(4);
                func.encode_impl(buffer);
                args.encode_impl(buffer);
                tail.encode_impl(buffer);
            },
            Bytecode::CallDynamic { func, args, tail } => {
                buffer.push(5);
                func.encode_impl(buffer);
                args.encode_impl(buffer);
                tail.encode_impl(buffer);
            },
            Bytecode::JumpIf { value, label } => {
                buffer.push(6);
                value.encode_impl(buffer);
                label.encode_impl(buffer);
            },
            Bytecode::InitOrJump { def_span, func, label } => {
                buffer.push(7);
                def_span.encode_impl(buffer);
                func.encode_impl(buffer);
                label.encode_impl(buffer);
            },
            Bytecode::Label(label) => {
                buffer.push(8);
                label.encode_impl(buffer);
            },
            Bytecode::Return(ssa) => {
                buffer.push(9);
                ssa.encode_impl(buffer);
            },
            Bytecode::Intrinsic { intrinsic, args, dst } => {
                buffer.push(10);
                intrinsic.encode_impl(buffer);
                args.encode_impl(buffer);
                dst.encode_impl(buffer);
            },
            Bytecode::InitTuple { elements, dst } => {
                buffer.push(11);
                elements.encode_impl(buffer);
                dst.encode_impl(buffer);
            },
            Bytecode::InitList { elements, dst } => {
                buffer.push(12);
                elements.encode_impl(buffer);
                dst.encode_impl(buffer);
            },
            Bytecode::PushDebugInfo { kind, src } => {
                buffer.push(13);
                kind.encode_impl(buffer);
                src.encode_impl(buffer);
            },
            Bytecode::PopDebugInfo => {
                buffer.push(14);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (value, cursor) = Value::decode_impl(buffer, cursor + 1)?;
                let (dst, cursor) = Memory::decode_impl(buffer, cursor)?;
                Ok((Bytecode::Const { value, dst }, cursor))
            },
            Some(1) => {
                let (src, cursor) = Memory::decode_impl(buffer, cursor + 1)?;
                let (dst, cursor) = Memory::decode_impl(buffer, cursor)?;
                Ok((Bytecode::Move { src, dst }, cursor))
            },
            Some(2) => {
                let (pair, cursor) = <(u32, u32)>::decode_impl(buffer, cursor + 1)?;
                let (dst, cursor) = Memory::decode_impl(buffer, cursor)?;
                Ok((Bytecode::Phi { pair, dst }, cursor))
            },
            Some(3) => {
                let (dst, cursor) = Label::decode_impl(buffer, cursor + 1)?;
                Ok((Bytecode::Jump(dst), cursor))
            },
            Some(4) => {
                let (func, cursor) = Label::decode_impl(buffer, cursor + 1)?;
                let (args, cursor) = Vec::<u32>::decode_impl(buffer, cursor)?;
                let (tail, cursor) = bool::decode_impl(buffer, cursor)?;
                Ok((Bytecode::Call { func, args, tail }, cursor))
            },
            Some(5) => {
                let (func, cursor) = Memory::decode_impl(buffer, cursor + 1)?;
                let (args, cursor) = Vec::<u32>::decode_impl(buffer, cursor)?;
                let (tail, cursor) = bool::decode_impl(buffer, cursor)?;
                Ok((Bytecode::CallDynamic { func, args, tail }, cursor))
            },
            Some(6) => {
                let (value, cursor) = Memory::decode_impl(buffer, cursor + 1)?;
                let (label, cursor) = Label::decode_impl(buffer, cursor)?;
                Ok((Bytecode::JumpIf { value, label }, cursor))
            },
            Some(7) => {
                let (def_span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                let (func, cursor) = Label::decode_impl(buffer, cursor)?;
                let (label, cursor) = Label::decode_impl(buffer, cursor)?;
                Ok((Bytecode::InitOrJump { def_span, func, label }, cursor))
            },
            Some(8) => {
                let (label, cursor) = Label::decode_impl(buffer, cursor + 1)?;
                Ok((Bytecode::Label(label), cursor))
            },
            Some(9) => {
                let (ssa, cursor) = u32::decode_impl(buffer, cursor + 1)?;
                Ok((Bytecode::Return(ssa), cursor))
            },
            Some(10) => {
                let (intrinsic, cursor) = Intrinsic::decode_impl(buffer, cursor + 1)?;
                let (args, cursor) = Vec::<u32>::decode_impl(buffer, cursor)?;
                let (dst, cursor) = Memory::decode_impl(buffer, cursor)?;
                Ok((Bytecode::Intrinsic { intrinsic, args, dst }, cursor))
            },
            Some(11) => {
                let (elements, cursor) = usize::decode_impl(buffer, cursor + 1)?;
                let (dst, cursor) = Memory::decode_impl(buffer, cursor)?;
                Ok((Bytecode::InitTuple { elements, dst }, cursor))
            },
            Some(12) => {
                let (elements, cursor) = usize::decode_impl(buffer, cursor + 1)?;
                let (dst, cursor) = Memory::decode_impl(buffer, cursor)?;
                Ok((Bytecode::InitList { elements, dst }, cursor))
            },
            Some(13) => {
                let (kind, cursor) = DebugInfoKind::decode_impl(buffer, cursor + 1)?;
                let (src, cursor) = Memory::decode_impl(buffer, cursor)?;
                Ok((Bytecode::PushDebugInfo { kind, src }, cursor))
            },
            Some(14) => Ok((Bytecode::PopDebugInfo, cursor + 1)),
            Some(n @ 15..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for DropType {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            DropType::Scalar => {
                buffer.push(0);
            },
            DropType::SimpleCompound => {
                buffer.push(1);
            },
            DropType::List(element) => {
                buffer.push(2);
                element.encode_impl(buffer);
            },
            DropType::Compound(elements) => {
                buffer.push(3);
                elements.encode_impl(buffer);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((DropType::Scalar, cursor + 1)),
            Some(1) => Ok((DropType::SimpleCompound, cursor + 1)),
            Some(2) => {
                let (element, cursor) = Box::<DropType>::decode_impl(buffer, cursor + 1)?;
                Ok((DropType::List(element), cursor))
            },
            Some(3) => {
                let (elements, cursor) = Vec::<DropType>::decode_impl(buffer, cursor + 1)?;
                Ok((DropType::Compound(elements), cursor))
            },
            Some(n @ 4..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
