use crate::{Bytecode, Label, Memory, Offset, Session, Value};
use sodigy_number::bi_to_string;
use std::fmt::{Display, Error, Formatter};

impl Display for Bytecode {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Bytecode::Const { dst, value } => write!(fmt, "{dst} = {value}"),
            Bytecode::Move { dst, src } => write!(fmt, "{dst} = {src}"),
            Bytecode::Read { src, offset, dst } => match offset {
                Offset::Static(i) => write!(fmt, "{dst} = @Deref({src})._{i}"),
                Offset::Dynamic(m) => write!(fmt, "{dst} = @Deref({src})[{m}]"),
            },
            Bytecode::IncStackPointer(n) => write!(fmt, "$sp += {n}"),
            Bytecode::DecStackPointer(n) => write!(fmt, "$sp -= {n}"),
            Bytecode::Jump(l) => write!(fmt, "@Jump({l})"),
            Bytecode::JumpIf { value, label } => write!(fmt, "@JumpIf({value}, {label})"),
            Bytecode::Intrinsic { intrinsic, stack_offset, dst } => write!(
                fmt,
                "{dst} = @{intrinsic:?}({})",
                (0..intrinsic.num_params()).map(
                    |i| format!("${}", i + *stack_offset)
                ).collect::<Vec<_>>().join(", "),
            ),
            Bytecode::InitTuple { stack_offset, elements, dst } => write!(
                fmt,
                "{dst} = @InitTuple({})",
                (0..*elements).map(
                    |i| format!("${}", i + *stack_offset)
                ).collect::<Vec<_>>().join(", "),
            ),
            _ => write!(fmt, "{self:?}"),
        }
    }
}

impl Display for Memory {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Memory::Return => write!(fmt, "$ret"),
            Memory::Stack(i) => write!(fmt, "${i}"),
            Memory::Global(s) => write!(fmt, "$g({:09x})", s.hash() & 0xfff_fff_fff),
        }
    }
}

impl Display for Value {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Value::Scalar(n) => write!(fmt, "%s({n})"),
            Value::Int(n) => write!(fmt, "%i({})", bi_to_string(n.is_neg, &n.nums)),
            Value::List(elems) => write!(
                fmt,
                "%l({})",
                elems.iter().map(
                    |elem| elem.to_string()
                ).collect::<Vec<_>>().join(", "),
            ),
            Value::Compound(elems) => write!(
                fmt,
                "%c({})",
                elems.iter().map(
                    |elem| elem.to_string()
                ).collect::<Vec<_>>().join(", "),
            ),
            _ => write!(fmt, "{self:?}"),
        }
    }
}

impl Display for Label {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Label::Local(n) => write!(fmt, "#L({n})"),
            Label::Global(s) => write!(fmt, "#G({:09x})", s.hash() & 0xfff_fff_fff),
            Label::Flatten(n) => write!(fmt, "#F({n})"),
        }
    }
}
