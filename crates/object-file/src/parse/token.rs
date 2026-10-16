use sodigy_error::Error;
use sodigy_number::BigInt;
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::Delim;

#[derive(Clone, Copy, Debug)]
pub enum Keyword {
    If,
    Else,
    Jump,
    Return,
    Label,
    With,
    Effect,
    Name,
    Fn,
    Code,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum TokenKind {
    Group {
        delim: Delim,
        tokens: Vec<Token>,
    },
    String(InternedString),
    Section(SectionName),
    GlobalValue(u128),
    InternedValue(u128),
    Scalar(u32),
    FuncPointer(u128),
    Integer(BigInt),
    Keyword(Keyword),
    GlobalLabel(u128),
    LocalLabel(u32),
    SSA(u32),
    Command(Command),
    Punct(u8),
}

#[derive(Clone, Copy, Debug)]
pub enum SectionName {
    Data,
    Code,
    Main,
    Assert,
}

#[derive(Clone, Copy, Debug)]
pub enum Command {
    NegInt,
    AddInt,
    BitOrScalar,
}

impl Command {
    pub fn from_ident(ident: &[u8]) -> Result<Command, Error> {
        match ident {
            b"NegInt" => Ok(Command::NegInt),
            b"AddInt" => Ok(Command::AddInt),
            b"BitOrScalar" => Ok(Command::BitOrScalar),
            // _ => panic!("{:?}", String::from_utf8_lossy(ident)),
            _ => todo!(),
        }
    }
}

