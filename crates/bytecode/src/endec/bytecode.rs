use crate::{
    Bytecode,
    DebugInfoKind,
    DropType,
    InPlaceOrMemory,
    Label,
    Memory,
    Offset,
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
            Bytecode::Update { src, offset, value, dst } => {
                buffer.push(2);
                src.encode_impl(buffer);
                offset.encode_impl(buffer);
                value.encode_impl(buffer);
                dst.encode_impl(buffer);
            },
            Bytecode::Read { src, offset, dst } => {
                buffer.push(3);
                src.encode_impl(buffer);
                offset.encode_impl(buffer);
                dst.encode_impl(buffer);
            },
            Bytecode::IncStackPointer(n) => {
                buffer.push(4);
                n.encode_impl(buffer);
            },
            Bytecode::DecStackPointer(n) => {
                buffer.push(5);
                n.encode_impl(buffer);
            },
            Bytecode::IncRefCount(dst) => {
                buffer.push(6);
                dst.encode_impl(buffer);
            },
            Bytecode::DecRefCount { dst, drop } => {
                buffer.push(7);
                dst.encode_impl(buffer);
                drop.encode_impl(buffer);
            },
            Bytecode::Jump(dst) => {
                buffer.push(8);
                dst.encode_impl(buffer);
            },
            Bytecode::JumpDynamic(dst) => {
                buffer.push(9);
                dst.encode_impl(buffer);
            },
            Bytecode::JumpIf { value, label } => {
                buffer.push(10);
                value.encode_impl(buffer);
                label.encode_impl(buffer);
            },
            Bytecode::JumpIfUninit { def_span, label } => {
                buffer.push(11);
                def_span.encode_impl(buffer);
                label.encode_impl(buffer);
            },
            Bytecode::Label(label) => {
                buffer.push(12);
                label.encode_impl(buffer);
            },
            Bytecode::PushCallStack(label) => {
                buffer.push(13);
                label.encode_impl(buffer);
            },
            Bytecode::PopCallStack => {
                buffer.push(14);
            },
            Bytecode::Return => {
                buffer.push(15);
            },
            Bytecode::Intrinsic { intrinsic, stack_offset, dst } => {
                buffer.push(16);
                intrinsic.encode_impl(buffer);
                stack_offset.encode_impl(buffer);
                dst.encode_impl(buffer);
            },
            Bytecode::InitTuple { stack_offset, elements, dst } => {
                buffer.push(17);
                stack_offset.encode_impl(buffer);
                elements.encode_impl(buffer);
                dst.encode_impl(buffer);
            },
            Bytecode::InitList { stack_offset, elements, dst } => {
                buffer.push(18);
                stack_offset.encode_impl(buffer);
                elements.encode_impl(buffer);
                dst.encode_impl(buffer);
            },
            Bytecode::PushDebugInfo { kind, src } => {
                buffer.push(19);
                kind.encode_impl(buffer);
                src.encode_impl(buffer);
            },
            Bytecode::PopDebugInfo => {
                buffer.push(20);
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
                let (src, cursor) = Memory::decode_impl(buffer, cursor + 1)?;
                let (offset, cursor) = Offset::decode_impl(buffer, cursor)?;
                let (value, cursor) = Memory::decode_impl(buffer, cursor)?;
                let (dst, cursor) = InPlaceOrMemory::decode_impl(buffer, cursor)?;
                Ok((Bytecode::Update { src, offset, value, dst }, cursor))
            },
            Some(3) => {
                let (src, cursor) = Memory::decode_impl(buffer, cursor + 1)?;
                let (offset, cursor) = Offset::decode_impl(buffer, cursor)?;
                let (dst, cursor) = Memory::decode_impl(buffer, cursor)?;
                Ok((Bytecode::Read { src, offset, dst }, cursor))
            },
            Some(4) => {
                let (n, cursor) = usize::decode_impl(buffer, cursor + 1)?;
                Ok((Bytecode::IncStackPointer(n), cursor))
            },
            Some(5) => {
                let (n, cursor) = usize::decode_impl(buffer, cursor + 1)?;
                Ok((Bytecode::DecStackPointer(n), cursor))
            },
            Some(6) => {
                let (dst, cursor) = Memory::decode_impl(buffer, cursor + 1)?;
                Ok((Bytecode::IncRefCount(dst), cursor))
            },
            Some(7) => {
                let (dst, cursor) = Memory::decode_impl(buffer, cursor + 1)?;
                let (drop, cursor) = DropType::decode_impl(buffer, cursor)?;
                Ok((Bytecode::DecRefCount { dst, drop }, cursor))
            },
            Some(8) => {
                let (dst, cursor) = Label::decode_impl(buffer, cursor + 1)?;
                Ok((Bytecode::Jump(dst), cursor))
            },
            Some(9) => {
                let (dst, cursor) = Memory::decode_impl(buffer, cursor + 1)?;
                Ok((Bytecode::JumpDynamic(dst), cursor))
            },
            Some(10) => {
                let (value, cursor) = Memory::decode_impl(buffer, cursor + 1)?;
                let (label, cursor) = Label::decode_impl(buffer, cursor)?;
                Ok((Bytecode::JumpIf { value, label }, cursor))
            },
            Some(11) => {
                let (def_span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                let (label, cursor) = Label::decode_impl(buffer, cursor)?;
                Ok((Bytecode::JumpIfUninit { def_span, label }, cursor))
            },
            Some(12) => {
                let (label, cursor) = Label::decode_impl(buffer, cursor + 1)?;
                Ok((Bytecode::Label(label), cursor))
            },
            Some(13) => {
                let (label, cursor) = Label::decode_impl(buffer, cursor + 1)?;
                Ok((Bytecode::PushCallStack(label), cursor))
            },
            Some(14) => Ok((Bytecode::PopCallStack, cursor + 1)),
            Some(15) => Ok((Bytecode::Return, cursor + 1)),
            Some(16) => {
                let (intrinsic, cursor) = Intrinsic::decode_impl(buffer, cursor + 1)?;
                let (stack_offset, cursor) = usize::decode_impl(buffer, cursor)?;
                let (dst, cursor) = Memory::decode_impl(buffer, cursor)?;
                Ok((Bytecode::Intrinsic { intrinsic, stack_offset, dst }, cursor))
            },
            Some(17) => {
                let (stack_offset, cursor) = usize::decode_impl(buffer, cursor + 1)?;
                let (elements, cursor) = usize::decode_impl(buffer, cursor)?;
                let (dst, cursor) = Memory::decode_impl(buffer, cursor)?;
                Ok((Bytecode::InitTuple { stack_offset, elements, dst }, cursor))
            },
            Some(18) => {
                let (stack_offset, cursor) = usize::decode_impl(buffer, cursor + 1)?;
                let (elements, cursor) = usize::decode_impl(buffer, cursor)?;
                let (dst, cursor) = Memory::decode_impl(buffer, cursor)?;
                Ok((Bytecode::InitList { stack_offset, elements, dst }, cursor))
            },
            Some(19) => {
                let (kind, cursor) = DebugInfoKind::decode_impl(buffer, cursor + 1)?;
                let (src, cursor) = Memory::decode_impl(buffer, cursor)?;
                Ok((Bytecode::PushDebugInfo { kind, src }, cursor))
            },
            Some(20) => Ok((Bytecode::PopDebugInfo, cursor + 1)),
            Some(n @ 21..) => Err(DecodeError::InvalidEnumVariant(*n)),
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
