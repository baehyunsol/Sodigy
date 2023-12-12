use crate::token::TokenKind;

#[derive(Clone, Copy)]
pub enum Flag {
    Output,
    From,
    To,
    Help,
    Version,
    ShowWarnings,
    SaveIr,
    DumpHir,
}

pub const FLAGS: [Flag; 8] = [
    Flag::Output,
    Flag::From,
    Flag::To,
    Flag::Help,
    Flag::Version,
    Flag::ShowWarnings,
    Flag::SaveIr,
    Flag::DumpHir,
];

impl Flag {
    /// what kind of param this flag takes
    pub fn param_type(&self) -> TokenKind {
        match self {
            Flag::Output => TokenKind::Path,
            Flag::From
            | Flag::To => TokenKind::Stage,
            Flag::ShowWarnings
            | Flag::SaveIr
            | Flag::DumpHir => TokenKind::Bool,
            Flag::Help
            | Flag::Version => TokenKind::None,
        }
    }

    pub fn short(&self) -> Option<&[u8]> {
        match self {
            Flag::Output => Some(b"-o"),
            Flag::From => Some(b"-f"),
            Flag::To => Some(b"-t"),
            Flag::Help => Some(b"-h"),
            Flag::Version => Some(b"-v"),
            Flag::ShowWarnings
            | Flag::SaveIr
            | Flag::DumpHir => None,
        }
    }

    pub fn long(&self) -> &[u8] {
        match self {
            Flag::Output => b"--output",
            Flag::From => b"--from",
            Flag::To => b"--to",
            Flag::Help => b"--help",
            Flag::Version => b"--version",
            Flag::ShowWarnings => b"--show-warnings",
            Flag::SaveIr => b"--save-ir",
            Flag::DumpHir => b"--dump-hir",
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
