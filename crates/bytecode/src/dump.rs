use crate::{
    Bytecode,
    GlobalLabel,
    InternedValue,
    LocalLabel,
    Memory,
    SSA,
    Value,
};
use sodigy_number::bi_to_hex_string;
use sodigy_span::Span;
use std::fmt::{Display, Error, Formatter};

impl Bytecode {
    pub fn dump(&self, debug_info_flag: bool) -> String {
        match self {
            Bytecode::Const { dst, value, debug_info } => format!(
                "{dst} = {value};{}",
                dump_debug_info(debug_info, debug_info_flag),
            ),
            Bytecode::Move { dst, src } => format!("{dst} = {src};"),
            Bytecode::Phi { pair: (x, y), dst } => format!("{dst} = $Phi({x}, {y});"),
            Bytecode::Jump(label) => format!("jump {label};"),
            Bytecode::Call { func, args, dst, debug_info, effect: _ } => format!(
                "{}call {func}({});{}",
                if let Some(dst) = dst { format!("{dst} = ") } else { String::from("return ") },
                args.iter().map(
                    |i| format!("{i}")
                ).collect::<Vec<_>>().join(", "),
                dump_debug_info(debug_info, debug_info_flag),
            ),
            Bytecode::CallDynamic { func, args, dst, debug_info, effect: _ } => format!(
                "{}dyn_call {func}({});{}",
                if let Some(dst) = dst { format!("{dst} = ") } else { String::from("return ") },
                args.iter().map(
                    |i| format!("{i}")
                ).collect::<Vec<_>>().join(", "),
                dump_debug_info(debug_info, debug_info_flag),
            ),
            Bytecode::JumpIf { value, t, f, debug_info } => format!(
                "if {value} {{ jump {t}; }} else {{ jump {f}; }}{}",
                dump_debug_info(debug_info, debug_info_flag),
            ),
            Bytecode::TryInitGlobal { global, label } => format!(
                "if !is_init(_g{}) {{ call {global}(); }} jump {label};",
                global.hex(20),
            ),
            Bytecode::LoadGlobal { src, dst } => format!(
                "{dst} = $LoadGlobal(_g{});",
                src.hex(20),
            ),
            Bytecode::StoreGlobal { src, dst } => format!(
                "$StoreGlobal({src}, _g{});",
                dst.hex(20),
            ),
            Bytecode::Label(label) => format!("label {label}:"),
            Bytecode::Return(ssa) => format!("return {ssa};"),
            Bytecode::Update { src, size: _, index, value, dst } => format!("{dst} = {src} `{index} {value};"),
            Bytecode::Intrinsic { intrinsic, args, dst, debug_info } => format!(
                "{dst} = ${intrinsic:?}({});{}",
                args.iter().map(
                    |i| format!("{i}")
                ).collect::<Vec<_>>().join(", "),
                dump_debug_info(debug_info, debug_info_flag),
            ),
            Bytecode::InitTuple { elements, dst, debug_info } => format!(
                "{dst} = $InitTuple({elements});{}",
                dump_debug_info(debug_info, debug_info_flag),
            ),
            Bytecode::InitList { elements, dst, debug_info } => format!(
                "{dst} = $InitList({elements});{}",
                dump_debug_info(debug_info, debug_info_flag),
            ),
            Bytecode::IncRefCount(memory) => format!("$IncRefCount({memory});"),
            Bytecode::DecRefCount(memory) => format!("$DecRefCount({memory});"),
            Bytecode::TryDrop(_, _) => format!("{self:?}"),
            Bytecode::Breakpoint => String::from("$Breakpoint;"),
        }
    }
}

impl Display for Bytecode {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{}", self.dump(true))
    }
}

impl Display for Memory {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Memory::SSA(i) => write!(fmt, "{i}"),
            Memory::Heap { ptr, offset } => match offset {
                0 => write!(fmt, "*{ptr}"),
                i => write!(fmt, "*({ptr} + {i})"),
            },
            Memory::List { ptr, offset } => write!(fmt, "{ptr}[{offset}]"),
        }
    }
}

impl Display for SSA {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "_{}", self.0)
    }
}

impl Display for InternedValue {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            InternedValue::Interned(h) => write!(fmt, "%I{}", h.hex(20)),
            InternedValue::Scalar(n) => write!(fmt, "{n:x}#s"),
            InternedValue::FuncPointer(def_span) => write!(fmt, "{}#f", def_span.hex(20)),
        }
    }
}

impl Display for LocalLabel {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "@L{:x}", self.index())
    }
}

impl Display for GlobalLabel {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "@G{}", self.hex(20))
    }
}

impl Display for Value {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Value::Scalar(n) => write!(fmt, "{n:x}#s"),
            Value::Int(n) => write!(fmt, "{}#n", bi_to_hex_string(n.is_neg, &n.nums)),
            Value::List(es) => write!(
                fmt,
                "[{}]",
                es.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(", "),
            ),
            Value::Compound(es) => write!(
                fmt,
                "{{{}}}",
                es.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(", "),
            ),
            Value::FuncPointer(def_span) => write!(fmt, "{}#f", def_span.hex(20)),
        }
    }
}

pub fn dump_debug_info(debug_info: &Option<Box<Span>>, flag: bool) -> String {
    match debug_info {
        Some(span) if **span == Span::None => String::new(),
        Some(span) if flag => format!("  // {span:?}"),
        _ => String::new(),
    }
}

