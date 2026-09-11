use super::Section;

#[derive(Debug)]
pub enum BytecodeParseError {
    UnexpectedByte {
        expected: Option<u8>,
        got: u8,
        cursor: usize,
    },
    FailedToParseHex {
        cursor: usize,
    },
    FailedToParseValue {
        cursor: usize,
    },
    MissingSection(Section),
    InvalidSection {
        cursor: usize,
    },
    DuplicateSection(Section),
}
