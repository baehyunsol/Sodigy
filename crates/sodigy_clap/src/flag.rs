use crate::token::TokenKind;

mod fmt;

#[derive(Clone, Copy, Debug)]
pub enum Flag {
    Output,
    To,
    Help,
    Version,
    ShowWarnings,
    SaveIr,
    SaveIrTo,
    DumpTokens,
    DumpTokensTo,
    DumpHir,
    DumpHirTo,
}

pub const FLAGS: [Flag; 11] = [
    Flag::Output,
    Flag::To,
    Flag::Help,
    Flag::Version,
    Flag::ShowWarnings,
    Flag::SaveIr,
    Flag::SaveIrTo,
    Flag::DumpTokens,
    Flag::DumpTokensTo,
    Flag::DumpHir,
    Flag::DumpHirTo,
];

impl Flag {
    /// what kind of param this flag takes
    pub fn param_type(&self) -> TokenKind {
        match self {
            Flag::Output
            | Flag::SaveIrTo
            | Flag::DumpTokensTo
            | Flag::DumpHirTo => TokenKind::Path,
            Flag::To => TokenKind::Stage,
            Flag::ShowWarnings
            | Flag::SaveIr
            | Flag::DumpTokens
            | Flag::DumpHir => TokenKind::Bool,
            Flag::Help
            | Flag::Version => TokenKind::None,
        }
    }

    pub fn short(&self) -> Option<&[u8]> {
        match self {
            Flag::Output => Some(b"-o"),
            Flag::To => Some(b"-t"),
            Flag::Help => Some(b"-h"),
            Flag::Version => Some(b"-v"),
            Flag::ShowWarnings
            | Flag::SaveIr
            | Flag::SaveIrTo
            | Flag::DumpTokens
            | Flag::DumpTokensTo
            | Flag::DumpHir
            | Flag::DumpHirTo => None,
        }
    }

    pub fn long(&self) -> &[u8] {
        match self {
            Flag::Output => b"--output",
            Flag::To => b"--to",
            Flag::Help => b"--help",
            Flag::Version => b"--version",
            Flag::ShowWarnings => b"--show-warnings",
            Flag::SaveIr => b"--save-ir",
            Flag::SaveIrTo => b"--save-ir-to",
            Flag::DumpTokens => b"--dump-tokens",
            Flag::DumpTokensTo => b"--dump-tokens-to",
            Flag::DumpHir => b"--dump-hir",
            Flag::DumpHirTo => b"--dump-hir-to",
        }
    }

    pub fn try_parse(s: &[u8]) -> Option<Self> {
        for flag in FLAGS.iter() {
            if let Some(short) = flag.short() {
                if s == short {
                    return Some(*flag);
                }
            }

            if s == flag.long() {
                return Some(*flag);
            }
        }

        None
    }
}
