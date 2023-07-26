use crate::session::{InternedString, LocalParseSession};
use crate::token::OpToken;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PrefixOp {
    Not,
    Neg,
}

impl From<&OpToken> for PrefixOp {
    fn from(t: &OpToken) -> PrefixOp {
        match t {
            OpToken::Sub => PrefixOp::Neg,
            OpToken::Not => PrefixOp::Not,
            _ => unreachable!("Internal Compiler Error 94FD57BFF22: {t:?}"),
        }
    }
}

impl From<&PrefixOp> for OpToken {
    fn from(op: &PrefixOp) -> OpToken {
        match op {
            PrefixOp::Not => OpToken::Not,
            PrefixOp::Neg => OpToken::Sub,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
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
    Index,  // `[]`
    Path,   // `.`
    Concat, // `<>`
    Range,  // `..`

    ModifyField(InternedString),  // $foo
}

impl InfixOp {
    pub fn dump(&self, session: &LocalParseSession) -> String {
        if let InfixOp::ModifyField(field) = self {
            format!("ModifyField({})", field.to_string(session))
        }
        else {
            format!("{self:?}")
        }
    }
}

impl From<&OpToken> for InfixOp {
    fn from(t: &OpToken) -> InfixOp {
        match t {
            OpToken::Add => InfixOp::Add,
            OpToken::Sub => InfixOp::Sub,
            OpToken::Mul => InfixOp::Mul,
            OpToken::Div => InfixOp::Div,
            OpToken::Rem => InfixOp::Rem,
            OpToken::Eq => InfixOp::Eq,
            OpToken::Gt => InfixOp::Gt,
            OpToken::Lt => InfixOp::Lt,
            OpToken::Ne => InfixOp::Ne,
            OpToken::Ge => InfixOp::Ge,
            OpToken::Le => InfixOp::Le,
            OpToken::And => InfixOp::BitwiseAnd,
            OpToken::AndAnd => InfixOp::LogicalAnd,
            OpToken::Or => InfixOp::BitwiseOr,
            OpToken::OrOr => InfixOp::LogicalOr,
            OpToken::Dot => InfixOp::Path,
            OpToken::Concat => InfixOp::Concat,
            OpToken::DotDot => InfixOp::Range,
            _ => unreachable!("Internal Compiler Error 01EC27D4304: {t:?}"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PostfixOp {
    Range, // `..`
}

impl From<&OpToken> for PostfixOp {
    fn from(t: &OpToken) -> PostfixOp {
        match t {
            OpToken::DotDot => PostfixOp::Range,
            _ => unreachable!("Internal Compiler Error 0BA3488EA42: {t:?}"),
        }
    }
}
