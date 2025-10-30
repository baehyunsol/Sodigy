use crate::{Block, CallArg, Expr, If, Match, StructInitField};
use sodigy_endec::{DecodeError, Endec};
use sodigy_name_analysis::IdentWithOrigin;
use sodigy_number::InternedNumber;
use sodigy_parse::Field;
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::{InfixOp, PostfixOp, PrefixOp};

impl Endec for Expr {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            Expr::Identifier(id) => {
                buffer.push(0);
                id.encode_impl(buffer);
            },
            Expr::Number { n, span } => {
                buffer.push(1);
                n.encode_impl(buffer);
                span.encode_impl(buffer);
            },
            Expr::String { binary, s, span } => {
                buffer.push(2);
                binary.encode_impl(buffer);
                s.encode_impl(buffer);
                span.encode_impl(buffer);
            },
            Expr::Char { binary, ch, span } => {
                buffer.push(3);
                binary.encode_impl(buffer);
                ch.encode_impl(buffer);
                span.encode_impl(buffer);
            },
            Expr::If(r#if) => {
                buffer.push(4);
                r#if.encode_impl(buffer);
            },
            Expr::Match(r#match) => {
                buffer.push(5);
                r#match.encode_impl(buffer);
            },
            Expr::Block(block) => {
                buffer.push(6);
                block.encode_impl(buffer);
            },
            Expr::Call { func, args } => {
                buffer.push(7);
                func.encode_impl(buffer);
                args.encode_impl(buffer);
            },
            Expr::Tuple { elements, group_span } => {
                buffer.push(8);
                elements.encode_impl(buffer);
                group_span.encode_impl(buffer);
            },
            Expr::List { elements, group_span } => {
                buffer.push(9);
                elements.encode_impl(buffer);
                group_span.encode_impl(buffer);
            },
            Expr::StructInit { r#struct, fields, group_span } => {
                buffer.push(10);
                r#struct.encode_impl(buffer);
                fields.encode_impl(buffer);
                group_span.encode_impl(buffer);
            },
            Expr::Path { lhs, fields } => {
                buffer.push(11);
                lhs.encode_impl(buffer);
                fields.encode_impl(buffer);
            },
            Expr::FieldModifier { fields, lhs, rhs } => {
                buffer.push(12);
                fields.encode_impl(buffer);
                lhs.encode_impl(buffer);
                rhs.encode_impl(buffer);
            },
            Expr::PrefixOp { op, op_span, rhs } => {
                buffer.push(13);
                op.encode_impl(buffer);
                op_span.encode_impl(buffer);
                rhs.encode_impl(buffer);
            },
            Expr::InfixOp { op, op_span, lhs, rhs } => {
                buffer.push(14);
                op.encode_impl(buffer);
                op_span.encode_impl(buffer);
                lhs.encode_impl(buffer);
                rhs.encode_impl(buffer);
            },
            Expr::PostfixOp { op, op_span, lhs } => {
                buffer.push(15);
                op.encode_impl(buffer);
                op_span.encode_impl(buffer);
                lhs.encode_impl(buffer);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (id, cursor) = IdentWithOrigin::decode_impl(buffer, cursor + 1)?;
                Ok((Expr::Identifier(id), cursor))
            },
            Some(1) => {
                let (n, cursor) = InternedNumber::decode_impl(buffer, cursor + 1)?;
                let (span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((Expr::Number { n, span }, cursor))
            },
            Some(2) => {
                let (binary, cursor) = bool::decode_impl(buffer, cursor + 1)?;
                let (s, cursor) = InternedString::decode_impl(buffer, cursor)?;
                let (span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((Expr::String { binary, s, span }, cursor))
            },
            Some(3) => {
                let (binary, cursor) = bool::decode_impl(buffer, cursor + 1)?;
                let (ch, cursor) = u32::decode_impl(buffer, cursor)?;
                let (span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((Expr::Char { binary, ch, span }, cursor))
            },
            Some(4) => {
                let (r#if, cursor) = If::decode_impl(buffer, cursor + 1)?;
                Ok((Expr::If(r#if), cursor))
            },
            Some(5) => {
                let (r#match, cursor) = Match::decode_impl(buffer, cursor + 1)?;
                Ok((Expr::Match(r#match), cursor))
            },
            Some(6) => {
                let (block, cursor) = Block::decode_impl(buffer, cursor + 1)?;
                Ok((Expr::Block(block), cursor))
            },
            Some(7) => {
                let (func, cursor) = Box::<Expr>::decode_impl(buffer, cursor + 1)?;
                let (args, cursor) = Vec::<CallArg>::decode_impl(buffer, cursor)?;
                Ok((Expr::Call { func, args }, cursor))
            },
            Some(8) => {
                let (elements, cursor) = Vec::<Expr>::decode_impl(buffer, cursor + 1)?;
                let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((Expr::Tuple { elements, group_span }, cursor))
            },
            Some(9) => {
                let (elements, cursor) = Vec::<Expr>::decode_impl(buffer, cursor + 1)?;
                let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((Expr::List { elements, group_span }, cursor))
            },
            Some(10) => {
                let (r#struct, cursor) = Box::<Expr>::decode_impl(buffer, cursor + 1)?;
                let (fields, cursor) = Vec::<StructInitField>::decode_impl(buffer, cursor)?;
                let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((Expr::StructInit { r#struct, fields, group_span }, cursor))
            },
            Some(11) => {
                let (lhs, cursor) = Box::<Expr>::decode_impl(buffer, cursor + 1)?;
                let (fields, cursor) = Vec::<Field>::decode_impl(buffer, cursor)?;
                Ok((Expr::Path { lhs, fields }, cursor))
            },
            Some(12) => {
                let (fields, cursor) = Vec::<(InternedString, Span)>::decode_impl(buffer, cursor + 1)?;
                let (lhs, cursor) = Box::<Expr>::decode_impl(buffer, cursor)?;
                let (rhs, cursor) = Box::<Expr>::decode_impl(buffer, cursor)?;
                Ok((Expr::FieldModifier { fields, lhs, rhs }, cursor))
            },
            Some(13) => {
                let (op, cursor) = PrefixOp::decode_impl(buffer, cursor + 1)?;
                let (op_span, cursor) = Span::decode_impl(buffer, cursor)?;
                let (rhs, cursor) = Box::<Expr>::decode_impl(buffer, cursor)?;
                Ok((Expr::PrefixOp { op, op_span, rhs }, cursor))
            },
            Some(14) => {
                let (op, cursor) = InfixOp::decode_impl(buffer, cursor + 1)?;
                let (op_span, cursor) = Span::decode_impl(buffer, cursor)?;
                let (lhs, cursor) = Box::<Expr>::decode_impl(buffer, cursor)?;
                let (rhs, cursor) = Box::<Expr>::decode_impl(buffer, cursor)?;
                Ok((Expr::InfixOp { op, op_span, lhs, rhs }, cursor))
            },
            Some(15) => {
                let (op, cursor) = PostfixOp::decode_impl(buffer, cursor + 1)?;
                let (op_span, cursor) = Span::decode_impl(buffer, cursor)?;
                let (lhs, cursor) = Box::<Expr>::decode_impl(buffer, cursor)?;
                Ok((Expr::PostfixOp { op, op_span, lhs }, cursor))
            },
            Some(n) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
