use crate::token::TokenKind;

mod fmt;

#[derive(Clone, Copy, Debug)]
pub enum Flag {
    Output,
    StopAt,
    Help,
    Version,
    ShowWarnings,
    SaveIr,
    DumpHirTo,
    DumpMirTo,
    Clean,
    Verbose,
    RawInput,
}

pub const FLAGS: [Flag; 11] = [
    Flag::Output,
    Flag::StopAt,
    Flag::Help,
    Flag::Version,
    Flag::ShowWarnings,
    Flag::SaveIr,
    Flag::DumpHirTo,
    Flag::DumpMirTo,
    Flag::Clean,
    Flag::Verbose,
    Flag::RawInput,
];

impl Flag {
    /// what kind of param this flag takes
    pub fn param_type(&self) -> TokenKind {
        match self {
            Flag::Output
            | Flag::DumpHirTo
            | Flag::DumpMirTo => TokenKind::Path,
            Flag::StopAt => TokenKind::Stage,
            Flag::ShowWarnings
            | Flag::SaveIr => TokenKind::Bool,
            Flag::Verbose => TokenKind::Int,
            Flag::RawInput => TokenKind::RawInput,
            Flag::Help
            | Flag::Version
            | Flag::Clean => TokenKind::None,
        }
    }

    pub fn short(&self) -> Option<&[u8]> {
        match self {
            Flag::Output => Some(b"-o"),
            Flag::Help => Some(b"-h"),
            Flag::Version => Some(b"-v"),
            Flag::StopAt
            | Flag::ShowWarnings
            | Flag::SaveIr
            | Flag::DumpHirTo
            | Flag::DumpMirTo
            | Flag::Clean
            | Flag::Verbose
            | Flag::RawInput => None,
        }
    }

    pub fn long(&self) -> &[u8] {
        match self {
            Flag::Output => b"--output",
            Flag::StopAt => b"--stop-at",
            Flag::Help => b"--help",
            Flag::Version => b"--version",
            Flag::ShowWarnings => b"--show-warnings",
            Flag::SaveIr => b"--save-ir",
            Flag::DumpHirTo => b"--dump-hir-to",
            Flag::DumpMirTo => b"--dump-mir-to",
            Flag::Clean => b"--clean",
            Flag::Verbose => b"--verbose",
            Flag::RawInput => b"--raw-input",
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
