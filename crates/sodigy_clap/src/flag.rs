use crate::token::TokenKind;

mod fmt;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
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
    DumpType,
    Verbose,
    OrPatternLimit,
}

pub const FLAGS: [Flag; 14] = [
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
    Flag::DumpType,
    Flag::Verbose,
    Flag::OrPatternLimit,
];

impl Flag {
    pub fn arg_kind(&self) -> TokenKind {
        match self {
            Flag::Help => TokenKind::None,
            Flag::Version => TokenKind::None,
            Flag::Output => TokenKind::Path,
            Flag::Hir => TokenKind::None,
            Flag::Mir => TokenKind::None,
            Flag::Library => TokenKind::Library,
            Flag::ShowWarnings => TokenKind::None,
            Flag::HideWarnings => TokenKind::None,
            Flag::RawInput => TokenKind::String,
            Flag::DumpHirTo => TokenKind::Path,
            Flag::DumpMirTo => TokenKind::Path,
            Flag::DumpType => TokenKind::DumpType,
            Flag::Verbose => TokenKind::Integer,
            Flag::OrPatternLimit => TokenKind::Integer,
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
            Flag::DumpType => None,
            Flag::Verbose => None,
            Flag::OrPatternLimit => None,
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
            Flag::DumpType => Some(b"--dump-type"),
            Flag::Verbose => Some(b"--verbose"),
            Flag::OrPatternLimit => Some(b"--or-pattern-limit"),
        }
    }

    // use this to format Flag
    pub fn long_or_short(&self) -> &[u8] {
        // either self.long or self.short must succeed
        self.long().unwrap_or_else(|| self.short().unwrap())
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
