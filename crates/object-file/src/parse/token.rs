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
    // intrinsics from here
    NegInt,
    AddInt,
    SubInt,
    MulInt,
    DivInt,
    RemInt,
    LtInt,
    EqInt,
    GtInt,
    LtScalar,
    EqScalar,
    GtScalar,
    BitOrScalar,
    Nop1,
    // intrinsics till here

    InitList,
    InitTuple,
    IsInit,
    Phi,
}

impl Command {
    pub fn from_ident(ident: &[u8]) -> Result<Command, Error> {
        match ident {
            b"NegInt" => Ok(Command::NegInt),
            b"AddInt" => Ok(Command::AddInt),
            b"SubInt" => Ok(Command::SubInt),
            b"MulInt" => Ok(Command::MulInt),
            b"DivInt" => Ok(Command::DivInt),
            b"RemInt" => Ok(Command::RemInt),
            b"LtInt" => Ok(Command::LtInt),
            b"EqInt" => Ok(Command::EqInt),
            b"GtInt" => Ok(Command::GtInt),
            b"LtScalar" => Ok(Command::LtScalar),
            b"EqScalar" => Ok(Command::EqScalar),
            b"GtScalar" => Ok(Command::GtScalar),
            b"BitOrScalar" => Ok(Command::BitOrScalar),
            b"Nop1" => Ok(Command::Nop1),

            b"InitList" => Ok(Command::InitList),
            b"InitTuple" => Ok(Command::InitTuple),
            b"IsInit" => Ok(Command::IsInit),
            b"Phi" => Ok(Command::Phi),
            _ => panic!("{:?}", String::from_utf8_lossy(ident)),
            _ => todo!(),
        }
    }
}

