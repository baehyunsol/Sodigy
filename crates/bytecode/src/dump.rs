use crate::{Bytecode, Label, Memory, Offset, SSA, Value};
use sodigy_number::bi_to_string;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::fmt::{Display, Error, Formatter};

pub fn dump_bytecodes(
    name: InternedString,
    span: &Span,
    keyword: &str,
    params: Option<usize>,
    bytecodes: &[Bytecode],
    indent: usize,
    intermediate_dir: &str,
) -> String {
    let mut lines = vec![
        format!("// name: {}", name.unintern_or_default(intermediate_dir)),
        format!("// span: {span:?}"),
        format!(
            "{keyword} @G{:09x}{}:",
            span.hash() & 0xfff_fff_fff,
            match params {
                Some(params) => format!("({})", (0..params).map(|i| format!("_{i}")).collect::<Vec<_>>().join(", ")),
                None => String::new(),
            },
        ),
        format!("{}label @start:", " ".repeat(indent)),
    ];

    for bytecode in bytecodes.iter() {
        match bytecode {
            Bytecode::Label(_) => {
                lines.push(format!("{}{bytecode}", " ".repeat(indent)));
            },
            _ => {
                lines.push(format!("{}{bytecode}", " ".repeat(indent * 2)));
            },
        }
    }

    lines.join("\n")
}

impl Display for SSA {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "_{}", self.0)
    }
}

impl Display for Bytecode {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        fn dump_debug_info(debug_info: &Option<Box<Span>>) -> String {
            match debug_info {
                Some(span) if **span == Span::None => String::new(),
                Some(span) => format!("  // {span:?}"),
                None => String::new(),
            }
        }

        match self {
            Bytecode::Const { dst, value, debug_info } => write!(
                fmt,
                "{dst} = {value};{}",
                dump_debug_info(debug_info),
            ),
            Bytecode::Move { dst, src } => write!(fmt, "{dst} = {src};"),
            Bytecode::Phi { pair: (x, y), dst } => write!(fmt, "{dst} = phi({x}, {y});"),
            Bytecode::Jump(label) => write!(fmt, "jump {label};"),
            Bytecode::Call { func, args, dst, debug_info, effect: _ } => write!(
                fmt,
                "{}call {func}({});{}",
                if let Some(dst) = dst { format!("{dst} = ") } else { String::from("return ") },
                args.iter().map(
                    |i| format!("{i}")
                ).collect::<Vec<_>>().join(", "),
                dump_debug_info(debug_info),
            ),
            Bytecode::CallDynamic { func, args, dst, debug_info, effect: _ } => write!(
                fmt,
                "{}dyn_call ({func})({});{}",
                if let Some(dst) = dst { format!("{dst} = ") } else { String::from("return ") },
                args.iter().map(
                    |i| format!("{i}")
                ).collect::<Vec<_>>().join(", "),
                dump_debug_info(debug_info),
            ),
            Bytecode::JumpIf { value, label, debug_info } => write!(
                fmt,
                "if {value} {{ jump {label}; }}{}",
                dump_debug_info(debug_info),
            ),
            Bytecode::InitOrJump { def_span, func, label } => write!(
                fmt,
                "if is_init(_g{:09x}) {{ jump {label}; }} else {{ call {func}(); }}",
                def_span.hash() & 0xfff_fff_fff,
            ),
            Bytecode::Label(label) => write!(fmt, "label {label}:"),
            Bytecode::Return(ssa) => write!(fmt, "return {ssa};"),
            Bytecode::Update { src, size: _, index, value, dst } => write!(
                fmt,
                "{dst} = {src} `{index} {value};",
            ),
            Bytecode::Intrinsic { intrinsic, args, dst, debug_info } => write!(
                fmt,
                "{dst} = intrinsic {intrinsic:?}({});{}",
                args.iter().map(
                    |i| format!("{i}")
                ).collect::<Vec<_>>().join(", "),
                dump_debug_info(debug_info),
            ),
            Bytecode::InitTuple { elements, dst, debug_info } => write!(
                fmt,
                "{dst} = intrinsic InitTuple({elements});{}",
                dump_debug_info(debug_info),
            ),
            Bytecode::InitList { elements, dst, debug_info } => write!(
                fmt,
                "{dst} = intrinsic InitList({elements});{}",
                dump_debug_info(debug_info),
            ),
            _ => write!(fmt, "{self:?}"),
        }
    }
}

impl Display for Memory {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Memory::Return => write!(fmt, "_ret"),
            Memory::SSA(i) => write!(fmt, "{i}"),
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
