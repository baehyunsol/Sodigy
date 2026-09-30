use crate::{
    BasicBlock,
    Bytecode,
    CodeKind,
    CodeSection,
    ExprHash,
    GlobalLabel,
    InternedValue,
    LocalLabel,
    Memory,
    ObjectFile,
    SSA,
    Terminator,
    Value,
};
use sodigy_number::bi_to_hex_string;
use sodigy_span::Span;
use std::fmt::{Display, Error, Formatter};

impl Display for ObjectFile {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        let mut labels = vec![];

        if let Some(main_entry) = &self.main_entry {
            labels.push(String::from("main:"));
            labels.push(format!("    @G{}", main_entry.hex(20)));
        }

        if !self.asserts.is_empty() {
            labels.push(String::from("asserts:"));

            for assert in self.asserts.iter() {
                labels.push(format!("    @G{}", assert.1.hex(20)));
            }
        }

        let mut data: Vec<(&ExprHash, &Value)> = self.data.iter().collect();
        data.sort_by_key(|(h, _)| **h);
        let data = data.iter().map(|(h, v)| format!("    %I{} = {v};", h.hex(20))).collect::<Vec<_>>();

        let mut code: Vec<(&GlobalLabel, &CodeSection)> = self.code.iter().collect();
        code.sort_by_key(|(g, _)| **g);
        let code = code.iter().map(|(_, c)| c.to_string()).collect::<Vec<_>>();

        write!(fmt, r#".data:
{}

.code:
{}

.label:
{}
"#,
            data.join("\n"),
            code.join("\n\n"),
            labels.join("\n"),
        )
    }
}

impl CodeSection {
    pub fn dump(
        &self,
        show_line_number: bool,
        highlight: Option<(LocalLabel, Highlight)>,
        context: Option<usize>,
        debug_info: bool,
    ) -> String {
        let mut lines = vec![];
        let mut highlighted_line_no: Option<usize> = None;

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
        lines.push(format!("#[name({})]", self.name));

        lines.push(format!(
            "code @G{}{}:",
            self.label.hex(20),
            match self.params {
                Some(params) => format!("({})", (0..params).map(|i| format!("_{i}")).collect::<Vec<_>>().join(", ")),
                None => String::new(),
            },
        ));

        let mut basic_blocks: Vec<(&LocalLabel, &BasicBlock)> = self.basic_blocks.iter().collect();
        basic_blocks.sort_by_key(|(label, _)| *label);

        for (label, basic_block) in basic_blocks.iter() {
            for (i, line) in basic_block.dump(false, 0, None, debug_info).lines().enumerate() {
                if line.is_empty() {
                    continue;
                }

                match highlight {
                    Some((h_label, Highlight::Label)) if h_label == **label && i == 0 => {
                        highlighted_line_no = Some(lines.len());
                    },
                    Some((h_label, Highlight::Bytecode(j))) if h_label == **label && i == j + 1 => {
                        highlighted_line_no = Some(lines.len());
                    },
                    _ => {},
                }

                lines.push(format!("    {line}"));
            }

            if let Some((h_label, Highlight::Terminator)) = highlight && h_label == **label {
                highlighted_line_no = Some(lines.len() - 1);
            }
        }

        lines = lines.iter().enumerate().map(
            |(i, line)| match (show_line_number, Some(i) == highlighted_line_no, highlight.is_some()) {
                (true, true, _) => format!(">>> {i:>3} | {line}"),
                (true, false, true) => format!("    {i:>3} | {line}"),
                (true, false, false) => format!("{i:>3} | {line}"),
                (false, true, _) => format!(">>> {line}"),
                (false, false, true) => format!("    {line}"),
                (false, false, false) => line.to_string(),
            }
        ).collect();

        let clamp = match (context, highlighted_line_no) {
            (None, _) | (_, None) => None,
            (Some(c), Some(h)) => {
                let mut start = h.max(c) - c;
                let mut end = (start + 2 * c).min(lines.len());

                if end - start < 2 * c {
                    start = end.max(2 * c) - 2 * c;
                }

                Some((start, end))
            },
        };

        if let Some((start, end)) = clamp {
            lines = lines[start..end].to_vec();
        }

        lines.join("\n")
    }
}

impl Display for CodeSection {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{}", self.dump(false, None, None, true))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Highlight {
    Label,
    Bytecode(usize),
    Terminator,
}

impl BasicBlock {
    pub fn dump(
        &self,
        show_line_number: bool,
        line_number_offset: usize,
        highlight: Option<Highlight>,
        debug_info: bool,
    ) -> String {
        let code = self.code.iter().map(
            |bytecode| format!("    {}", bytecode.dump(debug_info))
        ).collect::<Vec<_>>().join("\n");

        let lines = format!(r#"label {}:
{code}
    {}{}"#,
            self.label,
            self.terminator,
            dump_debug_info(&self.terminator_debug_info, debug_info),
        );

        if !show_line_number && highlight.is_none() {
            return lines;
        }

        let mut result = Vec::with_capacity(lines.len());
        let lines_count = lines.lines().count();

        for (i, line) in lines.lines().enumerate() {
            let prefix = match (show_line_number, highlight) {
                (true, Some(Highlight::Label)) if i == 0 => format!(">>> {:>3} | ", i + line_number_offset),
                (true, Some(Highlight::Bytecode(j))) if i == j + 1 => format!(">>> {:>3} | ", i + line_number_offset),
                (true, Some(Highlight::Terminator)) if i + 1 == lines_count => format!(">>> {:>3} | ", i + line_number_offset),
                (true, Some(Highlight::Label) | Some(Highlight::Bytecode(_)) | Some(Highlight::Terminator)) => format!("    {:>3} | ", i + line_number_offset),
                (true, None) => format!(" {:>3} | ", i + line_number_offset),
                (false, Some(Highlight::Label)) if i == 0 => format!(">>> "),
                (false, Some(Highlight::Bytecode(j))) if i == j + 1 => String::from(">>> "),
                (false, Some(Highlight::Terminator)) if i + 1 == lines_count => String::from(">>> "),
                (false, Some(Highlight::Label) | Some(Highlight::Bytecode(_)) | Some(Highlight::Terminator)) => String::from("    "),
                (false, None) => unreachable!(),
            };
            result.push(format!("{prefix}{line}"));
        }

        result.join("\n")
    }
}

impl Display for BasicBlock {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{}", self.dump(false, 0, None, true))
    }
}

impl Display for Terminator {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        match self {
            Terminator::Jump(label) => write!(fmt, "jump {label};"),
            Terminator::TailCall { func, args } => write!(
                fmt,
                "return {func}({});",
                args.iter().map(
                    |i| format!("{i}")
                ).collect::<Vec<_>>().join(", "),
            ),
            Terminator::TailCallDynamic { func, args } => write!(
                fmt,
                "return {func}({});",
                args.iter().map(
                    |i| format!("{i}")
                ).collect::<Vec<_>>().join(", "),
            ),
            Terminator::JumpIf { value, t, f } => write!(fmt, "if {value} {{ jump {t}; }} else {{ jump {f}; }}"),
            Terminator::TryInitGlobal { global, label } => write!(
                fmt,
                "if !is_init(_g{}) {{ call {global}(); }} jump {label};",
                global.hex(20),
            ),
            Terminator::Return(ssa) => write!(fmt, "return {ssa};"),
        }
    }
}

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

fn dump_debug_info(debug_info: &Option<Box<Span>>, flag: bool) -> String {
    match debug_info {
        Some(span) if **span == Span::None => String::new(),
        Some(span) if flag => format!("  // {span:?}"),
        _ => String::new(),
    }
}
