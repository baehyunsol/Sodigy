use crate::{BasicBlock, Code, CodeKind, ObjectFile, Terminator};
use sodigy_bytecode::{
    ExprHash,
    GlobalLabel,
    LocalLabel,
    Value,
    dump_debug_info,
};
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
                labels.push(format!("    @G{}", assert.label.hex(20)));
            }
        }

        let mut data: Vec<(&ExprHash, &Value)> = self.data.iter().collect();
        data.sort_by_key(|(h, _)| **h);
        let data = data.iter().map(|(h, v)| format!("    %I{} = {v};", h.hex(20))).collect::<Vec<_>>();

        let mut code: Vec<(&GlobalLabel, &Code)> = self.code.iter().collect();
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

impl Code {
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
                let end = (start + 2 * c).min(lines.len());

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

impl Display for Code {
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
