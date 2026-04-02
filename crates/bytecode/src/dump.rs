use crate::{Bytecode, Label, Memory, Offset, Value};
use sodigy_number::bi_to_string;
use std::fmt::{Display, Error, Formatter};

impl Display for Bytecode {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Bytecode::Const { dst, value } => write!(fmt, "{dst} = {value};"),
            Bytecode::Move { dst, src } => write!(fmt, "{dst} = {src};"),
            Bytecode::Phi { pair: (x, y), dst } => write!(fmt, "{dst} = phi(_{x}, _{y});"),
            Bytecode::Jump(label) => write!(fmt, "jump {label};"),
            Bytecode::Call { func, args, tail } => write!(
                fmt,
                "{}call {func}({});",
                if *tail { "tail " } else { "" },
                args.iter().map(
                    |i| format!("_{i}")
                ).collect::<Vec<_>>().join(", "),
            ),
            Bytecode::CallDynamic { func, args, tail } => write!(
                fmt,
                "{}dyn_call ({func})({});",
                if *tail { "tail " } else { "" },
                args.iter().map(
                    |i| format!("_{i}")
                ).collect::<Vec<_>>().join(", "),
            ),
            Bytecode::JumpIf { value, label } => write!(fmt, "if {value} {{ jump {label}; }}"),
            Bytecode::InitOrJump { def_span, func, label } => write!(
                fmt,
                "if is_init(_g{:09x}) {{ jump {label}; }} else {{ call {func}(); }}",
                def_span.hash() & 0xfff_fff_fff,
            ),
            Bytecode::Label(label) => write!(fmt, "label {label}:"),
            Bytecode::Return(ssa) => write!(fmt, "return _{ssa};"),
            Bytecode::Intrinsic { intrinsic, args, dst } => write!(
                fmt,
                "{dst} = intrinsic {intrinsic:?}({});",
                args.iter().map(
                    |i| format!("_{i}")
                ).collect::<Vec<_>>().join(", "),
            ),
            Bytecode::InitTuple { elements, dst } => write!(fmt, "{dst} = intrinsic InitTuple({elements});"),
            Bytecode::InitList { elements, dst } => write!(fmt, "{dst} = intrinsic InitList({elements});"),
            _ => write!(fmt, "{self:?}"),
        }
    }
}

impl Display for Memory {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Memory::Return => write!(fmt, "_ret"),
            Memory::SSA(i) => write!(fmt, "_{i}"),
            Memory::Heap { ptr, offset } => match offset {
                Offset::Static(0) => write!(fmt, "*{ptr}"),
                Offset::Static(i) => write!(fmt, "*({ptr} + {i})"),
                Offset::Dynamic(p) => write!(fmt, "*({ptr} + *({p}))"),
            },
            Memory::List { ptr, offset } => match offset {
                Offset::Static(i) => write!(fmt, "{ptr}[{i}]"),
                Offset::Dynamic(p) => write!(fmt, "{ptr}[{p}]"),
            },
            Memory::Global(s) => write!(fmt, "_g{:09x}", s.hash() & 0xfff_fff_fff),
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
            Value::FuncPointer { def_span, program_counter } => match program_counter {
                Some(pc) => write!(fmt, "%f(@F{pc})"),
                None => write!(fmt, "%f(@S{:09x})", def_span.hash() & 0xfff_fff_fff),
            },
            Value::Span(s) => write!(fmt, "%sp({:09x})", s.hash() & 0xfff_fff_fff),
        }
    }
}

impl Display for Label {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Label::Local(n) => write!(fmt, "@L{n}"),
            Label::Global(s) => write!(fmt, "@G{:09x}", s.hash() & 0xfff_fff_fff),
            Label::Flatten(n) => write!(fmt, "@F{n}"),
        }
    }
}
