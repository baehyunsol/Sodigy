use crate::{
    Block,
    CallArg,
    Expr,
    ExprOrString,
    If,
    Match,
    Path,
    StructInitField,
    Type,
};
use sodigy_endec::{DecodeError, Endec};
use sodigy_parse::Field;
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::{Constant, InfixOp, PostfixOp, PrefixOp};

impl Endec for Expr {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            Expr::Path(path) => {
                buffer.push(0);
                path.encode_impl(buffer);
            },
            Expr::Constant(constant) => {
                buffer.push(1);
                constant.encode_impl(buffer);
            },
            Expr::If(r#if) => {
                buffer.push(2);
                r#if.encode_impl(buffer);
            },
            Expr::Match(r#match) => {
                buffer.push(3);
                r#match.encode_impl(buffer);
            },
            Expr::Block(block) => {
                buffer.push(4);
                block.encode_impl(buffer);
            },
            Expr::Call { func, args, arg_group_span } => {
                buffer.push(5);
                func.encode_impl(buffer);
                args.encode_impl(buffer);
                arg_group_span.encode_impl(buffer);
            },
            Expr::FormattedString { raw, elements, span } => {
                buffer.push(6);
                raw.encode_impl(buffer);
                elements.encode_impl(buffer);
                span.encode_impl(buffer);
            },
            Expr::Tuple { elements, group_span } => {
                buffer.push(7);
                elements.encode_impl(buffer);
                group_span.encode_impl(buffer);
            },
            Expr::List { elements, group_span } => {
                buffer.push(8);
                elements.encode_impl(buffer);
                group_span.encode_impl(buffer);
            },
            Expr::StructInit { constructor, fields, group_span } => {
                buffer.push(9);
                constructor.encode_impl(buffer);
                fields.encode_impl(buffer);
                group_span.encode_impl(buffer);
            },
            Expr::Field { lhs, fields, types } => {
                buffer.push(10);
                lhs.encode_impl(buffer);
                fields.encode_impl(buffer);
                types.encode_impl(buffer);
            },
            Expr::FieldUpdate { fields, lhs, rhs } => {
                buffer.push(11);
                fields.encode_impl(buffer);
                lhs.encode_impl(buffer);
                rhs.encode_impl(buffer);
            },
            Expr::PrefixOp { op, op_span, rhs } => {
                buffer.push(12);
                op.encode_impl(buffer);
                op_span.encode_impl(buffer);
                rhs.encode_impl(buffer);
            },
            Expr::InfixOp { op, op_span, lhs, rhs } => {
                buffer.push(13);
                op.encode_impl(buffer);
                op_span.encode_impl(buffer);
                lhs.encode_impl(buffer);
                rhs.encode_impl(buffer);
            },
            Expr::PostfixOp { op, op_span, lhs } => {
                buffer.push(14);
                op.encode_impl(buffer);
                op_span.encode_impl(buffer);
                lhs.encode_impl(buffer);
            },
            Expr::Closure { fp, captures } => {
                buffer.push(15);
                fp.encode_impl(buffer);
                captures.encode_impl(buffer);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (path, cursor) = Path::decode_impl(buffer, cursor + 1)?;
                Ok((Expr::Path(path), cursor))
            },
            Some(1) => {
                let (constant, cursor) = Constant::decode_impl(buffer, cursor + 1)?;
                Ok((Expr::Constant(constant), cursor))
            },
            Some(2) => {
                let (r#if, cursor) = If::decode_impl(buffer, cursor + 1)?;
                Ok((Expr::If(r#if), cursor))
            },
            Some(3) => {
                let (r#match, cursor) = Match::decode_impl(buffer, cursor + 1)?;
                Ok((Expr::Match(r#match), cursor))
            },
            Some(4) => {
                let (block, cursor) = Block::decode_impl(buffer, cursor + 1)?;
                Ok((Expr::Block(block), cursor))
            },
            Some(5) => {
                let (func, cursor) = Box::<Expr>::decode_impl(buffer, cursor + 1)?;
                let (args, cursor) = Vec::<CallArg>::decode_impl(buffer, cursor)?;
                let (arg_group_span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((Expr::Call { func, args, arg_group_span }, cursor))
            },
            Some(6) => {
                let (raw, cursor) = bool::decode_impl(buffer, cursor + 1)?;
                let (elements, cursor) = Vec::<ExprOrString>::decode_impl(buffer, cursor)?;
                let (span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((Expr::FormattedString { raw, elements, span }, cursor))
            },
            Some(7) => {
                let (elements, cursor) = Vec::<Expr>::decode_impl(buffer, cursor + 1)?;
                let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((Expr::Tuple { elements, group_span }, cursor))
            },
            Some(8) => {
                let (elements, cursor) = Vec::<Expr>::decode_impl(buffer, cursor + 1)?;
                let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((Expr::List { elements, group_span }, cursor))
            },
            Some(9) => {
                let (constructor, cursor) = Path::decode_impl(buffer, cursor + 1)?;
                let (fields, cursor) = Vec::<StructInitField>::decode_impl(buffer, cursor)?;
                let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((Expr::StructInit { constructor, fields, group_span }, cursor))
            },
            Some(10) => {
                let (lhs, cursor) = Box::<Expr>::decode_impl(buffer, cursor + 1)?;
                let (fields, cursor) = Vec::<Field>::decode_impl(buffer, cursor)?;
                let (types, cursor) = Vec::<Option<Vec<Type>>>::decode_impl(buffer, cursor)?;
                Ok((Expr::Field { lhs, fields, types }, cursor))
            },
            Some(11) => {
                let (fields, cursor) = Vec::<Field>::decode_impl(buffer, cursor + 1)?;
                let (lhs, cursor) = Box::<Expr>::decode_impl(buffer, cursor)?;
                let (rhs, cursor) = Box::<Expr>::decode_impl(buffer, cursor)?;
                Ok((Expr::FieldUpdate { fields, lhs, rhs }, cursor))
            },
            Some(12) => {
                let (op, cursor) = PrefixOp::decode_impl(buffer, cursor + 1)?;
                let (op_span, cursor) = Span::decode_impl(buffer, cursor)?;
                let (rhs, cursor) = Box::<Expr>::decode_impl(buffer, cursor)?;
                Ok((Expr::PrefixOp { op, op_span, rhs }, cursor))
            },
            Some(13) => {
                let (op, cursor) = InfixOp::decode_impl(buffer, cursor + 1)?;
                let (op_span, cursor) = Span::decode_impl(buffer, cursor)?;
                let (lhs, cursor) = Box::<Expr>::decode_impl(buffer, cursor)?;
                let (rhs, cursor) = Box::<Expr>::decode_impl(buffer, cursor)?;
                Ok((Expr::InfixOp { op, op_span, lhs, rhs }, cursor))
            },
            Some(14) => {
                let (op, cursor) = PostfixOp::decode_impl(buffer, cursor + 1)?;
                let (op_span, cursor) = Span::decode_impl(buffer, cursor)?;
                let (lhs, cursor) = Box::<Expr>::decode_impl(buffer, cursor)?;
                Ok((Expr::PostfixOp { op, op_span, lhs }, cursor))
            },
            Some(15) => {
                let (fp, cursor) = Path::decode_impl(buffer, cursor + 1)?;
                let (captures, cursor) = Vec::<Span>::decode_impl(buffer, cursor)?;
                Ok((Expr::Closure { fp, captures }, cursor))
            },
            Some(n @ 16..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for ExprOrString {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            ExprOrString::Expr(e) => {
                buffer.push(0);
                e.encode_impl(buffer);
            },
            ExprOrString::String { s, span } => {
                buffer.push(1);
                s.encode_impl(buffer);
                span.encode_impl(buffer);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (e, cursor) = Expr::decode_impl(buffer, cursor + 1)?;
                Ok((ExprOrString::Expr(e), cursor))
            },
            Some(1) => {
                let (s, cursor) = InternedString::decode_impl(buffer, cursor + 1)?;
                let (span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((ExprOrString::String { s, span }, cursor))
            },
            Some(n @ 2..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
