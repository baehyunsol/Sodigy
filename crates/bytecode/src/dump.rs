use crate::{
    Bytecode,
    CodeKind,
    CodeSection,
    InternedValue,
    Label,
    Memory,
    ObjectFile,
    Offset,
    SSA,
    Value,
};
use sodigy_number::bi_to_string;
use sodigy_span::Span;
use std::fmt::{Display, Error, Formatter};

impl Display for CodeSection {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        let mut lines = vec![];

        if let Some(span) = &self.span {
            lines.push(format!("// span: {span:?}"));
        }

        let kind = match self.kind {
            CodeKind::Func => "func",
            CodeKind::Let => "global-let",
            CodeKind::Assert => "assertion",
        };
        lines.push(format!("// {kind}"));

        lines.push(format!("#[effect({})]", self.effect.single_word()));
        lines.push(format!("#[name({:?})]", self.name));

        // TODO: We're not supposed to use the term "data" here...
        lines.push(format!(
            "data @G{}{}:",
            self.label.hex(12),
            match self.params {
                Some(params) => format!("({})", (0..params).map(|i| format!("_{i}")).collect::<Vec<_>>().join(", ")),
                None => String::new(),
            },
        ));
        lines.push(String::from("    label @start:"));

        for bytecode in self.code.iter() {
            match bytecode {
                Bytecode::Label(_) => {
                    lines.push(format!("    {bytecode}"));
                },
                _ => {
                    lines.push(format!("        {bytecode}"));
                },
            }
        }

        write!(fmt, "{}", lines.join("\n"))
    }
}

impl Display for ObjectFile {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        let mut labels = vec![];

        if let Some(main_entry) = &self.main_entry {
            labels.push(String::from("main:"));
            labels.push(format!("    @G{}", main_entry.hex(12)));
        }

        if !self.asserts.is_empty() {
            labels.push(String::from("asserts:"));

            for assert in self.asserts.iter() {
                labels.push(format!("    @G{}", assert.hex(12)));
            }
        }

        write!(fmt, r#".data:
{}

.code:
{}

.label:
{}
"#,
            self.data.iter().map(|(h, v)| format!("%I{} = {v};", h.hex(12))).collect::<Vec<_>>().join("\n"),
            self.code.iter().map(|c| c.to_string()).collect::<Vec<_>>().join("\n\n"),
            labels.join("\n"),
        )
    }
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
            Bytecode::Phi { pair: (x, y), dst } => write!(fmt, "{dst} = $Phi({x}, {y});"),
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
                "if is_init(_g{}) {{ jump {label}; }} else {{ call {func}(); }}",
                def_span.hex(12),
            ),
            Bytecode::Label(label) => write!(fmt, "label {label}:"),
            Bytecode::Return(ssa) => write!(fmt, "return {ssa};"),
            Bytecode::Update { src, size: _, index, value, dst } => write!(
                fmt,
                "{dst} = {src} `{index} {value};",
            ),
            Bytecode::Intrinsic { intrinsic, args, dst, debug_info } => write!(
                fmt,
                "{dst} = ${intrinsic:?}({});{}",
                args.iter().map(
                    |i| format!("{i}")
                ).collect::<Vec<_>>().join(", "),
                dump_debug_info(debug_info),
            ),
            Bytecode::InitTuple { elements, dst, debug_info } => write!(
                fmt,
                "{dst} = $InitTuple({elements});{}",
                dump_debug_info(debug_info),
            ),
            Bytecode::InitList { elements, dst, debug_info } => write!(
                fmt,
                "{dst} = $InitList({elements});{}",
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
            Memory::Global(s) => write!(fmt, "_g{}", s.hex(12)),
        }
    }
}

impl Display for InternedValue {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            InternedValue::Interned(h) => write!(fmt, "%I{}", h.hex(12)),
            InternedValue::Scalar(n) => write!(fmt, "{n}#s"),
        }
    }
}

impl Display for Label {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Label::Local(n) => write!(fmt, "@L{n}"),
            Label::Global(s) => write!(fmt, "@G{}", s.hex(12)),
            Label::Flatten(n) => write!(fmt, "@F{n}"),
        }
    }
}

impl Display for Value {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Value::Scalar(n) => write!(fmt, "{n}#s"),
            Value::Int(n) => write!(fmt, "{}#n", bi_to_string(n.is_neg, &n.nums)),
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
            Value::FuncPointer { def_span, .. } => write!(fmt, "{}#f", def_span.hex(12)),

            // This information is lost once it's dumped.
            Value::Span(_) => write!(fmt, "#p"),
        }
    }
}
