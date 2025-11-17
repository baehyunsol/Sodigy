use crate::Punct;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum InfixOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Shl,
    Shr,
    Lt,
    Eq,
    Gt,
    Leq,
    Neq,
    Geq,
    Index,
    Concat,
    Append,
    Prepend,
    Range { inclusive: bool },
    BitAnd,
    BitOr,
    LogicAnd,
    LogicOr,
    Xor,
}

impl InfixOp {
    pub fn get_def_lang_item(&self) -> &'static str {
        match self {
            InfixOp::Add => "op.add",
            InfixOp::Sub => "op.sub",
            InfixOp::Mul => "op.mul",
            InfixOp::Div => "op.div",
            InfixOp::Rem => "op.rem",
            InfixOp::Shl => "op.shl",
            InfixOp::Shr => "op.shr",
            InfixOp::Lt => "op.lt",
            InfixOp::Eq => "op.eq",
            InfixOp::Gt => "op.gt",
            InfixOp::Leq => "op.leq",
            InfixOp::Neq => "op.neq",
            InfixOp::Geq => "op.geq",
            InfixOp::Index => "op.index",
            InfixOp::Concat => "op.concat",
            InfixOp::Append => "op.append",
            InfixOp::Prepend => "op.prepend",
            InfixOp::LogicAnd => "op.logic_and",
            InfixOp::LogicOr => "op.logic_or",
            _ => panic!("TODO: {self:?}"),
        }
    }

    pub fn get_generic_lang_items(&self) -> Vec<&'static str> {
        match self {
            InfixOp::Add => vec!["op.add.generic.0", "op.add.generic.1", "op.add.generic.2"],
            InfixOp::Sub => vec!["op.sub.generic.0", "op.sub.generic.1", "op.sub.generic.2"],
            InfixOp::Mul => vec!["op.mul.generic.0", "op.mul.generic.1", "op.mul.generic.2"],
            InfixOp::Div => vec!["op.div.generic.0", "op.div.generic.1", "op.div.generic.2"],
            InfixOp::Rem => vec!["op.rem.generic.0", "op.rem.generic.1", "op.rem.generic.2"],
            InfixOp::Shl => vec!["op.shl.generic.0", "op.shl.generic.1", "op.shl.generic.2"],
            InfixOp::Shr => vec!["op.shr.generic.0", "op.shr.generic.1", "op.shr.generic.2"],
            InfixOp::Lt => vec!["op.lt.generic.0"],
            InfixOp::Eq => vec!["op.eq.generic.0"],
            InfixOp::Gt => vec!["op.gt.generic.0"],
            _ => panic!("TODO: {self:?}"),
        }
    }
}

impl TryFrom<Punct> for InfixOp {
    type Error = ();

    fn try_from(p: Punct) -> Result<InfixOp, ()> {
        match p {
            Punct::Add => Ok(InfixOp::Add),
            Punct::Sub => Ok(InfixOp::Sub),
            Punct::Mul => Ok(InfixOp::Mul),
            Punct::Div => Ok(InfixOp::Div),
            Punct::Rem => Ok(InfixOp::Rem),
            Punct::Shl => Ok(InfixOp::Shl),
            Punct::Shr => Ok(InfixOp::Shr),
            Punct::Lt => Ok(InfixOp::Lt),
            Punct::Eq => Ok(InfixOp::Eq),
            Punct::Gt => Ok(InfixOp::Gt),
            Punct::Leq => Ok(InfixOp::Leq),
            Punct::Neq => Ok(InfixOp::Neq),
            Punct::Geq => Ok(InfixOp::Geq),
            Punct::And => Ok(InfixOp::BitAnd),
            Punct::Or => Ok(InfixOp::BitOr),
            Punct::Xor => Ok(InfixOp::Xor),
            Punct::AndAnd => Ok(InfixOp::LogicAnd),
            Punct::OrOr => Ok(InfixOp::LogicOr),
            Punct::Concat => Ok(InfixOp::Concat),
            Punct::Append => Ok(InfixOp::Append),
            Punct::Prepend => Ok(InfixOp::Prepend),
            Punct::DotDot => Ok(InfixOp::Range { inclusive: false }),
            Punct::DotDotEq => Ok(InfixOp::Range { inclusive: true }),
            // Do not use a wildcard!
            Punct::Colon | Punct::Semicolon | Punct::Assign |
            Punct::Comma | Punct::Dot | Punct::At | Punct::Dollar |
            Punct::Factorial | Punct::QuestionMark | Punct::Arrow |
            Punct::ReturnType => Err(()),
        }
    }
}
