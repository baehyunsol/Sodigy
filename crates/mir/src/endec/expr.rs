use crate::{
    Assert,
    Block,
    Callable,
    Expr,
    If,
    Let,
    Match,
    MatchArm,
    ShortCircuitKind,
};
use sodigy_endec::{DecodeError, Endec};
use sodigy_hir as hir;
use sodigy_name_analysis::IdentWithOrigin;
use sodigy_number::InternedNumber;
use sodigy_parse::Field;
use sodigy_span::Span;
use sodigy_string::InternedString;

impl Endec for Expr {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            Expr::Ident(id) => {
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
            Expr::Char { ch, span } => {
                buffer.push(3);
                ch.encode_impl(buffer);
                span.encode_impl(buffer);
            },
            Expr::Byte { b, span } => {
                buffer.push(4);
                b.encode_impl(buffer);
                span.encode_impl(buffer);
            },
            Expr::If(r#if) => {
                buffer.push(5);
                r#if.encode_impl(buffer);
            },
            Expr::Match(r#match) => {
                buffer.push(6);
                r#match.encode_impl(buffer);
            },
            Expr::Block(block) => {
                buffer.push(7);
                block.encode_impl(buffer);
            },
            Expr::Path { lhs, fields } => {
                buffer.push(8);
                lhs.encode_impl(buffer);
                fields.encode_impl(buffer);
            },
            Expr::FieldModifier { fields, lhs, rhs } => {
                buffer.push(9);
                fields.encode_impl(buffer);
                lhs.encode_impl(buffer);
                rhs.encode_impl(buffer);
            },
            Expr::Call { func, args, arg_group_span, generic_defs, given_keyword_arguments } => {
                buffer.push(10);
                func.encode_impl(buffer);
                args.encode_impl(buffer);
                arg_group_span.encode_impl(buffer);
                generic_defs.encode_impl(buffer);
                given_keyword_arguments.encode_impl(buffer);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (id, cursor) = IdentWithOrigin::decode_impl(buffer, cursor + 1)?;
                Ok((Expr::Ident(id), cursor))
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
                let (ch, cursor) = u32::decode_impl(buffer, cursor + 1)?;
                let (span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((Expr::Char { ch, span }, cursor))
            },
            Some(4) => {
                let (b, cursor) = u8::decode_impl(buffer, cursor + 1)?;
                let (span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((Expr::Byte { b, span }, cursor))
            },
            Some(5) => {
                let (r#if, cursor) = If::decode_impl(buffer, cursor + 1)?;
                Ok((Expr::If(r#if), cursor))
            },
            Some(6) => {
                let (r#match, cursor) = Match::decode_impl(buffer, cursor + 1)?;
                Ok((Expr::Match(r#match), cursor))
            },
            Some(7) => {
                let (block, cursor) = Block::decode_impl(buffer, cursor + 1)?;
                Ok((Expr::Block(block), cursor))
            },
            Some(8) => {
                let (lhs, cursor) = Box::<Expr>::decode_impl(buffer, cursor + 1)?;
                let (fields, cursor) = Vec::<Field>::decode_impl(buffer, cursor)?;
                Ok((Expr::Path { lhs, fields }, cursor))
            },
            Some(9) => {
                let (fields, cursor) = Vec::<(InternedString, Span)>::decode_impl(buffer, cursor + 1)?;
                let (lhs, cursor) = Box::<Expr>::decode_impl(buffer, cursor)?;
                let (rhs, cursor) = Box::<Expr>::decode_impl(buffer, cursor)?;
                Ok((Expr::FieldModifier { fields, lhs, rhs }, cursor))
            },
            Some(10) => {
                let (func, cursor) = Callable::decode_impl(buffer, cursor + 1)?;
                let (args, cursor) = Vec::<Expr>::decode_impl(buffer, cursor)?;
                let (arg_group_span, cursor) = Span::decode_impl(buffer, cursor)?;
                let (generic_defs, cursor) = Vec::<Span>::decode_impl(buffer, cursor)?;
                let (given_keyword_arguments, cursor) = Vec::<(InternedString, usize)>::decode_impl(buffer, cursor)?;
                Ok((Expr::Call { func, args, arg_group_span, generic_defs, given_keyword_arguments }, cursor))
            },
            Some(n @ 11..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for Block {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.group_span.encode_impl(buffer);
        self.lets.encode_impl(buffer);
        self.asserts.encode_impl(buffer);
        self.value.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (lets, cursor) = Vec::<Let>::decode_impl(buffer, cursor)?;
        let (asserts, cursor) = Vec::<Assert>::decode_impl(buffer, cursor)?;
        let (value, cursor) = Box::<Expr>::decode_impl(buffer, cursor)?;

        Ok((
            Block {
                group_span,
                lets,
                asserts,
                value,
            },
            cursor,
        ))
    }
}

impl Endec for If {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.if_span.encode_impl(buffer);
        self.cond.encode_impl(buffer);
        self.else_span.encode_impl(buffer);
        self.true_value.encode_impl(buffer);
        self.true_group_span.encode_impl(buffer);
        self.false_value.encode_impl(buffer);
        self.false_group_span.encode_impl(buffer);
        self.from_short_circuit.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (if_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (cond, cursor) = Box::<Expr>::decode_impl(buffer, cursor)?;
        let (else_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (true_value, cursor) = Box::<Expr>::decode_impl(buffer, cursor)?;
        let (true_group_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (false_value, cursor) = Box::<Expr>::decode_impl(buffer, cursor)?;
        let (false_group_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (from_short_circuit, cursor) = Option::<ShortCircuitKind>::decode_impl(buffer, cursor)?;

        Ok((
            If {
                if_span,
                cond,
                else_span,
                true_value,
                true_group_span,
                false_value,
                false_group_span,
                from_short_circuit,
            },
            cursor,
        ))
    }
}

impl Endec for Match {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.keyword_span.encode_impl(buffer);
        self.scrutinee.encode_impl(buffer);
        self.arms.encode_impl(buffer);
        self.group_span.encode_impl(buffer);
        self.lowered_from_if.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (keyword_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (scrutinee, cursor) = Box::<Expr>::decode_impl(buffer, cursor)?;
        let (arms, cursor) = Vec::<MatchArm>::decode_impl(buffer, cursor)?;
        let (group_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (lowered_from_if, cursor) = bool::decode_impl(buffer, cursor)?;

        Ok((
            Match {
                keyword_span,
                scrutinee,
                arms,
                group_span,
                lowered_from_if,
            },
            cursor,
        ))
    }
}

impl Endec for MatchArm {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.pattern.encode_impl(buffer);
        self.guard.encode_impl(buffer);
        self.value.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (pattern, cursor) = hir::Pattern::decode_impl(buffer, cursor)?;
        let (guard, cursor) = Option::<Expr>::decode_impl(buffer, cursor)?;
        let (value, cursor) = Expr::decode_impl(buffer, cursor)?;

        Ok((
            MatchArm {
                pattern,
                guard,
                value,
            },
            cursor,
        ))
    }
}

impl Endec for Callable {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            Callable::Static { def_span, span } => {
                buffer.push(0);
                def_span.encode_impl(buffer);
                span.encode_impl(buffer);
            },
            Callable::StructInit { def_span, span } => {
                buffer.push(1);
                def_span.encode_impl(buffer);
                span.encode_impl(buffer);
            },
            Callable::TupleInit { group_span } => {
                buffer.push(2);
                group_span.encode_impl(buffer);
            },
            Callable::ListInit { group_span } => {
                buffer.push(3);
                group_span.encode_impl(buffer);
            },
            Callable::Dynamic(f) => {
                buffer.push(4);
                f.encode_impl(buffer);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (def_span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                let (span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((Callable::Static { def_span, span }, cursor))
            },
            Some(1) => {
                let (def_span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                let (span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((Callable::StructInit { def_span, span }, cursor))
            },
            Some(2) => {
                let (group_span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                Ok((Callable::TupleInit { group_span }, cursor))
            },
            Some(3) => {
                let (group_span, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                Ok((Callable::ListInit { group_span }, cursor))
            },
            Some(4) => {
                let (f, cursor) = Box::<Expr>::decode_impl(buffer, cursor + 1)?;
                Ok((Callable::Dynamic(f), cursor))
            },
            Some(n @ 5..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for ShortCircuitKind {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            ShortCircuitKind::And => {
                buffer.push(0);
            },
            ShortCircuitKind::Or => {
                buffer.push(1);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((ShortCircuitKind::And, cursor + 1)),
            Some(1) => Ok((ShortCircuitKind::Or, cursor + 1)),
            Some(n @ 2..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
