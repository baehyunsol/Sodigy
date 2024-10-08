use smallvec::SmallVec;
use sodigy_parse::{IdentWithSpan, Punct};

mod endec;
mod fmt;

#[derive(Clone, Copy, Debug)]
pub enum PrefixOp {
    Not,
    Neg,
}

impl TryFrom<Punct> for PrefixOp {
    type Error = ();

    fn try_from(p: Punct) -> Result<Self, ()> {
        match p {
            Punct::Sub => Ok(PrefixOp::Neg),
            Punct::Not => Ok(PrefixOp::Not),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PostfixOp {
    Range,
    QuestionMark,
}

impl TryFrom<Punct> for PostfixOp {
    type Error = ();

    fn try_from(p: Punct) -> Result<Self, ()> {
        match p {
            Punct::DotDot => Ok(PostfixOp::Range),
            Punct::QuestionMark => Ok(PostfixOp::QuestionMark),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum InfixOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Gt,
    Lt,
    Ne,
    Ge,
    Le,
    BitwiseAnd,
    BitwiseOr,
    LogicalAnd,
    LogicalOr,
    Xor,
    ShiftRight,
    ShiftLeft,

    /// `[]`
    Index,

    /// `<>`
    Concat,

    /// `..`
    Range,

    /// `..~`
    InclusiveRange,

    /// `` ` ``
    FieldModifier(SmallVec<[IdentWithSpan; 2]>),
}

impl TryFrom<Punct> for InfixOp {
    type Error = ();

    fn try_from(p: Punct) -> Result<Self, ()> {
        match p {
            Punct::Add => Ok(InfixOp::Add),
            Punct::Sub => Ok(InfixOp::Sub),
            Punct::Mul => Ok(InfixOp::Mul),
            Punct::Div => Ok(InfixOp::Div),
            Punct::Rem => Ok(InfixOp::Rem),
            Punct::Concat => Ok(InfixOp::Concat),
            Punct::Eq => Ok(InfixOp::Eq),
            Punct::Gt => Ok(InfixOp::Gt),
            Punct::Lt => Ok(InfixOp::Lt),
            Punct::Ne => Ok(InfixOp::Ne),
            Punct::Ge => Ok(InfixOp::Ge),
            Punct::Le => Ok(InfixOp::Le),
            Punct::AndAnd => Ok(InfixOp::LogicalAnd),
            Punct::OrOr => Ok(InfixOp::LogicalOr),
            Punct::And => Ok(InfixOp::BitwiseAnd),
            Punct::Or => Ok(InfixOp::BitwiseOr),
            Punct::Xor => Ok(InfixOp::Xor),
            Punct::GtGt => Ok(InfixOp::ShiftRight),
            Punct::LtLt => Ok(InfixOp::ShiftLeft),
            Punct::DotDot => Ok(InfixOp::Range),
            Punct::InclusiveRange => Ok(InfixOp::InclusiveRange),
            Punct::FieldModifier(id) => Ok(InfixOp::FieldModifier(id)),

            // Don't use wildcard here
            Punct::At | Punct::Not
            | Punct::Assign | Punct::Comma
            | Punct::Dot | Punct::Colon
            | Punct::SemiColon
            | Punct::Backslash
            | Punct::Dollar
            | Punct::Backtick
            | Punct::QuestionMark
            | Punct::RArrow => Err(()),
        }
    }
}

impl TryFrom<PostfixOp> for InfixOp {
    type Error = ();

    fn try_from(op: PostfixOp) -> Result<Self, ()> {
        match op {
            PostfixOp::Range => Ok(InfixOp::Range),
            _ => Err(())
        }
    }
}

pub(crate) fn postfix_binding_power(op: PostfixOp) -> u32 {
    match op {
        PostfixOp::Range => RANGE,
        PostfixOp::QuestionMark => QUESTION,
    }
}

pub(crate) fn prefix_binding_power(op: PrefixOp) -> u32 {
    match op {
        PrefixOp::Not | PrefixOp::Neg => NEG,
    }
}

/// ref: https://doc.rust-lang.org/reference/expressions.html#expression-precedence\
/// ref: https://hexdocs.pm/elixir/main/operators.html\
pub fn infix_binding_power(op: InfixOp) -> (u32, u32) {
    match op {
        InfixOp::Add | InfixOp::Sub => (ADD, ADD + 1),
        InfixOp::Mul | InfixOp::Div | InfixOp::Rem => (MUL, MUL + 1),
        InfixOp::Concat => (CONCAT, CONCAT + 1),
        InfixOp::Range | InfixOp::InclusiveRange => (RANGE, RANGE + 1),
        InfixOp::Index => (INDEX, INDEX + 1),
        InfixOp::Gt | InfixOp::Lt | InfixOp::Ge | InfixOp::Le => (COMP, COMP + 1),
        InfixOp::Eq | InfixOp::Ne => (COMP_EQ, COMP_EQ + 1),
        InfixOp::ShiftRight | InfixOp::ShiftLeft => (SHIFT, SHIFT + 1),
        InfixOp::BitwiseAnd => (BITWISE_AND, BITWISE_AND + 1),
        InfixOp::Xor => (XOR, XOR + 1),
        InfixOp::BitwiseOr => (BITWISE_OR, BITWISE_OR + 1),
        InfixOp::FieldModifier(_) => (MODIFY, MODIFY + 1),
        InfixOp::LogicalAnd => (LOGICAL_AND, LOGICAL_AND + 1),
        InfixOp::LogicalOr => (LOGICAL_OR, LOGICAL_OR + 1),
    }
}

pub fn call_binding_power() -> (u32, u32) {
    (CALL, CALL + 1)
}

pub fn path_binding_power() -> (u32, u32) {
    (PATH, PATH + 1)
}

pub fn index_binding_power() -> (u32, u32) {
    (INDEX, INDEX + 1)
}

pub fn struct_init_binding_power() -> (u32, u32) {
    (STRUCT_INIT, STRUCT_INIT + 1)
}

const PATH: u32 = 37;
const STRUCT_INIT: u32 = 35;
const CALL: u32 = 33;
const INDEX: u32 = 31;
const QUESTION: u32 = 29;
const NEG: u32 = 27;
const MUL: u32 = 25;
const ADD: u32 = 23;
const SHIFT: u32 = 21;
const BITWISE_AND: u32 = 19;
const XOR: u32 = 17;
const BITWISE_OR: u32 = 15;
const CONCAT: u32 = 11; const RANGE: u32 = 11;
const COMP: u32 = 9;
const COMP_EQ: u32 = 7;
const MODIFY: u32 = 5;
const LOGICAL_AND: u32 = 3;
const LOGICAL_OR: u32 = 1;
