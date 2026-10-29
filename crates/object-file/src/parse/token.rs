use sodigy_error::Error;
use sodigy_mir::Intrinsic;
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
    Int(BigInt),
    Keyword(Keyword),
    GlobalLabel(u128),
    LocalLabel(u32),
    SSA(u32),
    Command(Command),
    Punct(u8),
}

impl TokenKind {
    pub fn render(&self) -> String {
        match self {
            TokenKind::Section(s) => format!(".{s:?}").to_ascii_lowercase(),
            TokenKind::Command(c) => format!("${c:?}"),
            TokenKind::Punct(p) => format!("{}", *p as char),
            _ => panic!("TODO: {self:?}"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    Update,
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
            b"Update" => Ok(Command::Update),
            b"IsInit" => Ok(Command::IsInit),
            b"Phi" => Ok(Command::Phi),
            b"LoadGlobal" => Ok(Command::LoadGlobal),
            b"StoreGlobal" => Ok(Command::StoreGlobal),
            b"Breakpoint" => Ok(Command::Breakpoint),
            _ => panic!("{:?}", String::from_utf8_lossy(ident)),
            _ => todo!(),
        }
    }

    pub fn try_into_intrinsic(&self) -> Option<Intrinsic> {
        match self {
            Command::NegInt => Some(Intrinsic::NegInt),
            Command::AddInt => Some(Intrinsic::AddInt),
            Command::SubInt => Some(Intrinsic::SubInt),
            Command::MulInt => Some(Intrinsic::MulInt),
            Command::DivInt => Some(Intrinsic::DivInt),
            Command::RemInt => Some(Intrinsic::RemInt),
            Command::LtInt => Some(Intrinsic::LtInt),
            Command::EqInt => Some(Intrinsic::EqInt),
            Command::GtInt => Some(Intrinsic::GtInt),
            Command::BitAndInt => Some(Intrinsic::BitAndInt),
            Command::BitOrInt => Some(Intrinsic::BitOrInt),
            Command::ShrInt => Some(Intrinsic::ShrInt),
            Command::ShlInt => Some(Intrinsic::ShlInt),
            Command::Ilog2Int => Some(Intrinsic::Ilog2Int),
            Command::AddScalar => Some(Intrinsic::AddScalar),
            Command::SubScalar => Some(Intrinsic::SubScalar),
            Command::MulScalar => Some(Intrinsic::MulScalar),
            Command::DivScalar => Some(Intrinsic::DivScalar),
            Command::RemScalar => Some(Intrinsic::RemScalar),
            Command::LtScalar => Some(Intrinsic::LtScalar),
            Command::EqScalar => Some(Intrinsic::EqScalar),
            Command::GtScalar => Some(Intrinsic::GtScalar),
            Command::BitAndScalar => Some(Intrinsic::BitAndScalar),
            Command::BitOrScalar => Some(Intrinsic::BitOrScalar),
            Command::ScalarToInt => Some(Intrinsic::ScalarToInt),
            Command::IntToScalar => Some(Intrinsic::IntToScalar),
            Command::IndexList => Some(Intrinsic::IndexList),
            Command::LenList => Some(Intrinsic::LenList),
            Command::SliceList => Some(Intrinsic::SliceList),
            Command::SliceRightList => Some(Intrinsic::SliceRightList),
            Command::AppendList => Some(Intrinsic::AppendList),
            Command::PrependList => Some(Intrinsic::PrependList),
            Command::Exit => Some(Intrinsic::Exit),
            Command::Print => Some(Intrinsic::Print),
            Command::EPrint => Some(Intrinsic::EPrint),
            Command::Debug => Some(Intrinsic::Debug),
            Command::RandomInt => Some(Intrinsic::RandomInt),
            Command::Sleep => Some(Intrinsic::Sleep),
            Command::Nop0 => Some(Intrinsic::Nop0),
            Command::Nop1 => Some(Intrinsic::Nop1),
            _ => None,
        }
    }
}

