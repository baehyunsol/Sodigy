use crate::token::TokenKind;

mod fmt;

#[derive(Clone, Copy, Debug)]
pub enum Flag {
    Help,
    Version,
    Output,
    Hir,
    Mir,
    Library,
    ShowWarnings,
    HideWarnings,
    RawInput,
    DumpHirTo,
    DumpMirTo,
    Verbose,
}

pub const FLAGS: [Flag; 12] = [
    Flag::Help,
    Flag::Version,
    Flag::Output,
    Flag::Hir,
    Flag::Mir,
    Flag::Library,
    Flag::ShowWarnings,
    Flag::HideWarnings,
    Flag::RawInput,
    Flag::DumpHirTo,
    Flag::DumpMirTo,
    Flag::Verbose,
];

impl Flag {
    pub fn arg_kind(&self) -> ArgKind {
        match self {
            Flag::Help => ArgKind::None,
            Flag::Version => ArgKind::None,
            Flag::Output => ArgKind::Path,
            Flag::Hir => ArgKind::None,
            Flag::Mir => ArgKind::None,
            Flag::Library => ArgKind::Library,
            Flag::ShowWarnings => ArgKind::None,
            Flag::HideWarnings => ArgKind::None,
            Flag::RawInput => ArgKind::String,
            Flag::DumpHirTo => ArgKind::Path,
            Flag::DumpMirTo => ArgKind::Path,
            Flag::Verbose => ArgKind::Integer,
        }
    }

    pub fn short(&self) -> Option<&[u8]> {
        match self {
            Flag::Help => Some(b"-h"),
            Flag::Version => Some(b"-v"),
            Flag::Output => Some(b"-o"),
            Flag::Hir => Some(b"-H"),
            Flag::Mir => Some(b"-M"),
            Flag::Library => Some(b"-L"),
            Flag::ShowWarnings => None,
            Flag::HideWarnings => None,
            Flag::RawInput => None,
            Flag::DumpHirTo => None,
            Flag::DumpMirTo => None,
            Flag::Verbose => None,
        }
    }

    pub fn long(&self) -> Option<&[u8]> {
        match self {
            Flag::Help => Some(b"--help"),
            Flag::Version => Some(b"--version"),
            Flag::Output => Some(b"--output"),
            Flag::Hir => Some(b"--hir"),
            Flag::Mir => Some(b"--mir"),
            Flag::Library => None,
            Flag::ShowWarnings => Some(b"--show-warnings"),
            Flag::HideWarnings => Some(b"--hide-warnings"),
            Flag::RawInput => Some(b"--raw-input"),
            Flag::DumpHirTo => Some(b"--dump-hir-to"),
            Flag::DumpMirTo => Some(b"--dump-mir-to"),
            Flag::Verbose => Some(b"--verbose"),
        }
    }

    pub fn try_parse(s: &[u8]) -> Option<Self> {
        for flag in FLAGS.iter() {
            if let Some(short) = flag.short() {
                if s == short {
                    return Some(*flag);
                }
            }

            if let Some(long) = flag.long() {
                if s == long {
                    return Some(*flag);
                }
            }
        }

        None
    }
}
