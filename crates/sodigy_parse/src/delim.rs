use sodigy_span::SpanRange;

mod endec;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Delim {
    Brace,    // {}
    Bracket,  // []
    Paren,    // ()
}

impl Delim {
    pub fn start(&self) -> u8 {
        match self {
            Delim::Brace => b'{',
            Delim::Bracket => b'[',
            Delim::Paren => b'(',
        }
    }

    pub fn end(&self) -> u8 {
        match self {
            Delim::Brace => b'}',
            Delim::Bracket => b']',
            Delim::Paren => b')',
        }
    }
}

impl From<u8> for Delim {
    // Don't call this function unless you're sure that `c` is valid
    fn from(c: u8) -> Self {
        match c {
            b'{' | b'}' => Delim::Brace,
            b'[' | b']' => Delim::Bracket,
            b'(' | b')' => Delim::Paren,
            _ => unreachable!(),
        }
    }
}

// used to find a match of a delim
pub struct DelimStart {
    pub(crate) kind: Delim,
    pub(crate) index: usize,
    pub(crate) prefix: u8,

    // span of the starting token
    // it's used for error messages
    pub(crate) span: SpanRange,
}

impl DelimStart {
    pub fn new(ch: u8, index: usize, span: SpanRange) -> Self {
        DelimStart {
            kind: ch.into(),
            index, span,
            prefix: b'\0'
        }
    }

    pub fn new_prefix(ch: u8, index: usize, span: SpanRange, prefix: u8) -> Self {
        DelimStart {
            kind: ch.into(),
            index, span,
            prefix,
        }
    }

    pub fn start_char(&self) -> u8 {
        self.kind.start()
    }
}
