use super::Section;

#[derive(Debug)]
pub enum BytecodeParseError {
    UnexpectedByte {
        expected: Option<u8>,
        got: u8,
        cursor: usize,
    },
    UnexpectedEnd,
    FailedToParseHex {
        cursor: usize,
    },
    FailedToParseValue {
        cursor: usize,
    },
    FailedToParseIdent {
        cursor: usize,
    },
    IntRangeError {
        cursor: usize,
    },
    MissingSection(Section),
    InvalidSection {
        cursor: usize,
    },
    DuplicateSection(Section),
}
