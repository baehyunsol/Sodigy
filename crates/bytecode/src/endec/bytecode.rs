use crate::{
    Bytecode,
    DropType,
    GlobalLabel,
    InternedValue,
    LocalLabel,
    Memory,
    SSA,
};
use sodigy_endec::{DecodeError, Endec};
use sodigy_error::FuncEffect;
use sodigy_mir::Intrinsic;
use sodigy_span::Span;

impl Endec for Bytecode {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            Bytecode::Const { value, dst, debug_info } => {
                buffer.push(0);
                value.encode_impl(buffer);
                dst.encode_impl(buffer);
                debug_info.encode_impl(buffer);
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
            Bytecode::Call { func, args, dst, debug_info, effect } => {
                buffer.push(4);
                func.encode_impl(buffer);
                args.encode_impl(buffer);
                dst.encode_impl(buffer);
                debug_info.encode_impl(buffer);
                effect.encode_impl(buffer);
            },
            Bytecode::CallDynamic { func, args, dst, debug_info, effect } => {
                buffer.push(5);
                func.encode_impl(buffer);
                args.encode_impl(buffer);
                dst.encode_impl(buffer);
                debug_info.encode_impl(buffer);
                effect.encode_impl(buffer);
            },
            Bytecode::JumpIf { value, t, f, debug_info } => {
                buffer.push(6);
                value.encode_impl(buffer);
                t.encode_impl(buffer);
                f.encode_impl(buffer);
                debug_info.encode_impl(buffer);
            },
            Bytecode::TryInitGlobal { global, label } => {
                buffer.push(7);
                global.encode_impl(buffer);
                label.encode_impl(buffer);
            },
            Bytecode::LoadGlobal { src, dst } => {
                buffer.push(8);
                src.encode_impl(buffer);
                dst.encode_impl(buffer);
            },
            Bytecode::StoreGlobal { src, dst } => {
                buffer.push(9);
                src.encode_impl(buffer);
                dst.encode_impl(buffer);
            },
            Bytecode::Label(label) => {
                buffer.push(10);
                label.encode_impl(buffer);
            },
            Bytecode::Return(ssa) => {
                buffer.push(11);
                ssa.encode_impl(buffer);
            },
            Bytecode::Update { src, size, index, value, dst } => {
                buffer.push(12);
                src.encode_impl(buffer);
                size.encode_impl(buffer);
                index.encode_impl(buffer);
                value.encode_impl(buffer);
                dst.encode_impl(buffer);
            },
            Bytecode::Intrinsic { intrinsic, args, dst, debug_info } => {
                buffer.push(13);
                intrinsic.encode_impl(buffer);
                args.encode_impl(buffer);
                dst.encode_impl(buffer);
                debug_info.encode_impl(buffer);
            },
            Bytecode::InitTuple { elements, dst, debug_info } => {
                buffer.push(14);
                elements.encode_impl(buffer);
                dst.encode_impl(buffer);
                debug_info.encode_impl(buffer);
            },
            Bytecode::InitList { elements, dst, debug_info } => {
                buffer.push(15);
                elements.encode_impl(buffer);
                dst.encode_impl(buffer);
                debug_info.encode_impl(buffer);
            },
            Bytecode::IncRefCount(memory) => {
                buffer.push(16);
                memory.encode_impl(buffer);
            },
            Bytecode::DecRefCount(memory) => {
                buffer.push(17);
                memory.encode_impl(buffer);
            },
            Bytecode::TryDrop(memory, drop_type) => {
                buffer.push(18);
                memory.encode_impl(buffer);
                drop_type.encode_impl(buffer);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (value, cursor) = InternedValue::decode_impl(buffer, cursor + 1)?;
                let (dst, cursor) = Memory::decode_impl(buffer, cursor)?;
                let (debug_info, cursor) = Option::<Box<Span>>::decode_impl(buffer, cursor)?;
                Ok((Bytecode::Const { value, dst, debug_info }, cursor))
            },
            Some(1) => {
                let (src, cursor) = Memory::decode_impl(buffer, cursor + 1)?;
                let (dst, cursor) = Memory::decode_impl(buffer, cursor)?;
                Ok((Bytecode::Move { src, dst }, cursor))
            },
            Some(2) => {
                let (pair, cursor) = <(SSA, SSA)>::decode_impl(buffer, cursor + 1)?;
                let (dst, cursor) = Memory::decode_impl(buffer, cursor)?;
                Ok((Bytecode::Phi { pair, dst }, cursor))
            },
            Some(3) => {
                let (dst, cursor) = LocalLabel::decode_impl(buffer, cursor + 1)?;
                Ok((Bytecode::Jump(dst), cursor))
            },
            Some(4) => {
                let (func, cursor) = GlobalLabel::decode_impl(buffer, cursor + 1)?;
                let (args, cursor) = Vec::<SSA>::decode_impl(buffer, cursor)?;
                let (dst, cursor) = Option::<Memory>::decode_impl(buffer, cursor)?;
                let (debug_info, cursor) = Option::<Box<Span>>::decode_impl(buffer, cursor)?;
                let (effect, cursor) = Box::<FuncEffect>::decode_impl(buffer, cursor)?;
                Ok((Bytecode::Call { func, args, dst, debug_info, effect }, cursor))
            },
            Some(5) => {
                let (func, cursor) = SSA::decode_impl(buffer, cursor + 1)?;
                let (args, cursor) = Vec::<SSA>::decode_impl(buffer, cursor)?;
                let (dst, cursor) = Option::<Memory>::decode_impl(buffer, cursor)?;
                let (debug_info, cursor) = Option::<Box<Span>>::decode_impl(buffer, cursor)?;
                let (effect, cursor) = Box::<FuncEffect>::decode_impl(buffer, cursor)?;
                Ok((Bytecode::CallDynamic { func, args, dst, debug_info, effect }, cursor))
            },
            Some(6) => {
                let (value, cursor) = SSA::decode_impl(buffer, cursor + 1)?;
                let (t, cursor) = LocalLabel::decode_impl(buffer, cursor)?;
                let (f, cursor) = LocalLabel::decode_impl(buffer, cursor)?;
                let (debug_info, cursor) = Option::<Box<Span>>::decode_impl(buffer, cursor)?;
                Ok((Bytecode::JumpIf { value, t, f, debug_info }, cursor))
            },
            Some(7) => {
                let (global, cursor) = GlobalLabel::decode_impl(buffer, cursor + 1)?;
                let (label, cursor) = LocalLabel::decode_impl(buffer, cursor)?;
                Ok((Bytecode::TryInitGlobal { global, label }, cursor))
            },
            Some(8) => {
                let (src, cursor) = GlobalLabel::decode_impl(buffer, cursor + 1)?;
                let (dst, cursor) = SSA::decode_impl(buffer, cursor)?;
                Ok((Bytecode::LoadGlobal { src, dst }, cursor))
            },
            Some(9) => {
                let (src, cursor) = SSA::decode_impl(buffer, cursor + 1)?;
                let (dst, cursor) = GlobalLabel::decode_impl(buffer, cursor)?;
                Ok((Bytecode::StoreGlobal { src, dst }, cursor))
            },
            Some(10) => {
                let (label, cursor) = LocalLabel::decode_impl(buffer, cursor + 1)?;
                Ok((Bytecode::Label(label), cursor))
            },
            Some(11) => {
                let (ssa, cursor) = SSA::decode_impl(buffer, cursor + 1)?;
                Ok((Bytecode::Return(ssa), cursor))
            },
            Some(12) => {
                let (src, cursor) = SSA::decode_impl(buffer, cursor + 1)?;
                let (size, cursor) = usize::decode_impl(buffer, cursor)?;
                let (index, cursor) = usize::decode_impl(buffer, cursor)?;
                let (value, cursor) = SSA::decode_impl(buffer, cursor)?;
                let (dst, cursor) = Memory::decode_impl(buffer, cursor)?;
                Ok((Bytecode::Update { src, size, index, value, dst }, cursor))
            },
            Some(13) => {
                let (intrinsic, cursor) = Intrinsic::decode_impl(buffer, cursor + 1)?;
                let (args, cursor) = Vec::<SSA>::decode_impl(buffer, cursor)?;
                let (dst, cursor) = Memory::decode_impl(buffer, cursor)?;
                let (debug_info, cursor) = Option::<Box<Span>>::decode_impl(buffer, cursor)?;
                Ok((Bytecode::Intrinsic { intrinsic, args, dst, debug_info }, cursor))
            },
            Some(14) => {
                let (elements, cursor) = usize::decode_impl(buffer, cursor + 1)?;
                let (dst, cursor) = Memory::decode_impl(buffer, cursor)?;
                let (debug_info, cursor) = Option::<Box<Span>>::decode_impl(buffer, cursor)?;
                Ok((Bytecode::InitTuple { elements, dst, debug_info }, cursor))
            },
            Some(15) => {
                let (elements, cursor) = usize::decode_impl(buffer, cursor + 1)?;
                let (dst, cursor) = Memory::decode_impl(buffer, cursor)?;
                let (debug_info, cursor) = Option::<Box<Span>>::decode_impl(buffer, cursor)?;
                Ok((Bytecode::InitList { elements, dst, debug_info }, cursor))
            },
            Some(16) => {
                let (memory, cursor) = Memory::decode_impl(buffer, cursor + 1)?;
                Ok((Bytecode::IncRefCount(memory), cursor))
            },
            Some(17) => {
                let (memory, cursor) = Memory::decode_impl(buffer, cursor + 1)?;
                Ok((Bytecode::DecRefCount(memory), cursor))
            },
            Some(18) => {
                let (memory, cursor) = Memory::decode_impl(buffer, cursor + 1)?;
                let (drop_type, cursor) = DropType::decode_impl(buffer, cursor)?;
                Ok((Bytecode::TryDrop(memory, drop_type), cursor))
            },
            Some(n @ 19..) => Err(DecodeError::InvalidEnumVariant(*n)),
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
