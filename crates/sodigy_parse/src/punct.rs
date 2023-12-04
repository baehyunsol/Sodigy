use sodigy_intern::InternedString;

mod endec;
mod fmt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Punct {
    At, // `@`
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Not,    // `!`
    Concat, // `<>`
    Assign, // `=`
    Eq,
    Gt,
    Lt,
    Ne,
    Ge,
    Le,
    GtGt,
    LtLt,
    And,
    AndAnd,
    Or,
    OrOr,
    Xor,
    Comma,
    Dot,
    Colon,
    SemiColon,
    DotDot,
    Backslash,
    Dollar,
    Backtick,
    QuestionMark,

    /// `..~`
    InclusiveRange,

    /// `=>`
    RArrow,

    /// `<+`
    Append,

    /// `+>`
    Prepend,

    /// ``` `field_name ```
    FieldModifier(InternedString),
}

impl Punct {
    pub fn try_from_two_chars(c1: u8, c2: u8) -> Option<Self> {
        match (c1, c2) {
            (b'<', b'>') => Some(Punct::Concat),
            (b'=', b'=') => Some(Punct::Eq),
            (b'!', b'=') => Some(Punct::Ne),
            (b'>', b'=') => Some(Punct::Ge),
            (b'<', b'=') => Some(Punct::Le),
            (b'>', b'>') => Some(Punct::GtGt),
            (b'<', b'<') => Some(Punct::LtLt),
            (b'&', b'&') => Some(Punct::AndAnd),
            (b'|', b'|') => Some(Punct::OrOr),
            (b'.', b'.') => Some(Punct::DotDot),
            (b'=', b'>') => Some(Punct::RArrow),
            (b'<', b'+') => Some(Punct::Append),
            (b'+', b'>') => Some(Punct::Prepend),
            _ => None,
        }
    }
}

impl TryFrom<u8> for Punct {
    type Error = ();

    fn try_from(c: u8) -> Result<Punct, Self::Error> {
        match c {
            b'@' => Ok(Punct::At),
            b'+' => Ok(Punct::Add),
            b'-' => Ok(Punct::Sub),
            b'*' => Ok(Punct::Mul),
            b'/' => Ok(Punct::Div),
            b'%' => Ok(Punct::Rem),
            b'!' => Ok(Punct::Not),
            b'=' => Ok(Punct::Assign),
            b'>' => Ok(Punct::Gt),
            b'<' => Ok(Punct::Lt),
            b'&' => Ok(Punct::And),
            b'|' => Ok(Punct::Or),
            b'^' => Ok(Punct::Xor),
            b',' => Ok(Punct::Comma),
            b'.' => Ok(Punct::Dot),
            b':' => Ok(Punct::Colon),
            b';' => Ok(Punct::SemiColon),
            b'\\' => Ok(Punct::Backslash),
            b'$' => Ok(Punct::Dollar),
            b'`' => Ok(Punct::Backtick),
            b'?' => Ok(Punct::QuestionMark),
            _ => Err(()),
        }
    }
}
