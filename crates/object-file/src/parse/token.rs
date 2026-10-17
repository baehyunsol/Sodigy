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
    Ndet,
    NdetFn,
    Proc,
    NdetProc,
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
    Asserts,
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
    BitAndInt,
    BitOrInt,
    ShrInt,
    ShlInt,
    Ilog2Int,
    AddScalar,
    SubScalar,
    MulScalar,
    DivScalar,
    RemScalar,
    LtScalar,
    EqScalar,
    GtScalar,
    BitAndScalar,
    BitOrScalar,
    ScalarToInt,
    IntToScalar,
    IndexList,
    LenList,
    SliceList,
    SliceRightList,
    AppendList,
    PrependList,
    Exit,
    Print,
    EPrint,
    Debug,
    RandomInt,
    Sleep,
    Nop0,
    Nop1,
    // intrinsics till here

    InitList,
    InitTuple,
    IsInit,
    Phi,
    LoadGlobal,
    StoreGlobal,
    Breakpoint,
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
            b"BitAndInt" => Ok(Command::BitAndInt),
            b"BitOrInt" => Ok(Command::BitOrInt),
            b"ShrInt" => Ok(Command::ShrInt),
            b"ShlInt" => Ok(Command::ShlInt),
            b"Ilog2Int" => Ok(Command::Ilog2Int),
            b"AddScalar" => Ok(Command::AddScalar),
            b"SubScalar" => Ok(Command::SubScalar),
            b"MulScalar" => Ok(Command::MulScalar),
            b"DivScalar" => Ok(Command::DivScalar),
            b"RemScalar" => Ok(Command::RemScalar),
            b"LtScalar" => Ok(Command::LtScalar),
            b"EqScalar" => Ok(Command::EqScalar),
            b"GtScalar" => Ok(Command::GtScalar),
            b"BitAndScalar" => Ok(Command::BitAndScalar),
            b"BitOrScalar" => Ok(Command::BitOrScalar),
            b"ScalarToInt" => Ok(Command::ScalarToInt),
            b"IntToScalar" => Ok(Command::IntToScalar),
            b"IndexList" => Ok(Command::IndexList),
            b"LenList" => Ok(Command::LenList),
            b"SliceList" => Ok(Command::SliceList),
            b"SliceRightList" => Ok(Command::SliceRightList),
            b"AppendList" => Ok(Command::AppendList),
            b"PrependList" => Ok(Command::PrependList),
            b"Exit" => Ok(Command::Exit),
            b"Print" => Ok(Command::Print),
            b"EPrint" => Ok(Command::EPrint),
            b"Debug" => Ok(Command::Debug),
            b"RandomInt" => Ok(Command::RandomInt),
            b"Sleep" => Ok(Command::Sleep),
            b"Nop0" => Ok(Command::Nop0),
            b"Nop1" => Ok(Command::Nop1),
            b"InitList" => Ok(Command::InitList),
            b"InitTuple" => Ok(Command::InitTuple),
            b"IsInit" => Ok(Command::IsInit),
            b"Phi" => Ok(Command::Phi),
            b"LoadGlobal" => Ok(Command::LoadGlobal),
            b"StoreGlobal" => Ok(Command::StoreGlobal),
            b"Breakpoint" => Ok(Command::Breakpoint),
            _ => panic!("{:?}", String::from_utf8_lossy(ident)),
            _ => todo!(),
        }
    }
}

