use crate::{LogEntry, LogId};
use sodigy_endec::{DecodeError, Endec};
use sodigy_error::{Error, ItemKind};
use sodigy_hir::{Expr, Path, Pattern, Type, Use};
use sodigy_span::Span;
use sodigy_string::InternedString;

impl Endec for LogId {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.0.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (id, cursor) = u32::decode_impl(buffer, cursor)?;
        Ok((LogId(id), cursor))
    }
}

impl Endec for LogEntry {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            LogEntry::ResolveAliasStart { id } => {
                buffer.push(0);
                id.encode_impl(buffer);
            },
            LogEntry::ResolveAliasEnd { id, has_error, last_errors } => {
                buffer.push(1);
                id.encode_impl(buffer);
                has_error.encode_impl(buffer);
                last_errors.encode_impl(buffer);
            },
            LogEntry::ResolveAliasLoopStart(n) => {
                buffer.push(2);
                n.encode_impl(buffer);
            },
            LogEntry::ResolveAliasLoopEnd(n) => {
                buffer.push(3);
                n.encode_impl(buffer);
            },
            LogEntry::ResolveItemStart { id, kind, name, span } => {
                buffer.push(4);
                id.encode_impl(buffer);
                kind.encode_impl(buffer);
                name.encode_impl(buffer);
                span.encode_impl(buffer);
            },
            LogEntry::ResolveItemEnd { id, has_error, last_errors } => {
                buffer.push(5);
                id.encode_impl(buffer);
                has_error.encode_impl(buffer);
                last_errors.encode_impl(buffer);
            },
            LogEntry::ResolveUseStart { id, r#use } => {
                buffer.push(6);
                id.encode_impl(buffer);
                r#use.encode_impl(buffer);
            },
            LogEntry::ResolveUseEnd { id, r#use, has_error, last_errors } => {
                buffer.push(7);
                id.encode_impl(buffer);
                r#use.encode_impl(buffer);
                has_error.encode_impl(buffer);
                last_errors.encode_impl(buffer);
            },
            LogEntry::ResolvePathStart { id, path, type_args } => {
                buffer.push(8);
                id.encode_impl(buffer);
                path.encode_impl(buffer);
                type_args.encode_impl(buffer);
            },
            LogEntry::ResolvePathEnd { id, path, has_error, last_errors } => {
                buffer.push(9);
                id.encode_impl(buffer);
                path.encode_impl(buffer);
                has_error.encode_impl(buffer);
                last_errors.encode_impl(buffer);
            },
            LogEntry::ResolveExprStart { id, expr } => {
                buffer.push(10);
                id.encode_impl(buffer);
                expr.encode_impl(buffer);
            },
            LogEntry::ResolveExprEnd { id, expr, has_error, last_errors } => {
                buffer.push(11);
                id.encode_impl(buffer);
                expr.encode_impl(buffer);
                has_error.encode_impl(buffer);
                last_errors.encode_impl(buffer);
            },
            LogEntry::ResolvePatternStart { id, pattern } => {
                buffer.push(12);
                id.encode_impl(buffer);
                pattern.encode_impl(buffer);
            },
            LogEntry::ResolvePatternEnd { id, pattern, has_error, last_errors } => {
                buffer.push(13);
                id.encode_impl(buffer);
                pattern.encode_impl(buffer);
                has_error.encode_impl(buffer);
                last_errors.encode_impl(buffer);
            },
            LogEntry::ResolveTypeStart { id, r#type } => {
                buffer.push(14);
                id.encode_impl(buffer);
                r#type.encode_impl(buffer);
            },
            LogEntry::ResolveTypeEnd { id, r#type, has_error, last_errors } => {
                buffer.push(15);
                id.encode_impl(buffer);
                r#type.encode_impl(buffer);
                has_error.encode_impl(buffer);
                last_errors.encode_impl(buffer);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (id, cursor) = LogId::decode_impl(buffer, cursor + 1)?;
                Ok((LogEntry::ResolveAliasStart { id }, cursor))
            },
            Some(1) => {
                let (id, cursor) = LogId::decode_impl(buffer, cursor + 1)?;
                let (has_error, cursor) = bool::decode_impl(buffer, cursor)?;
                let (last_errors, cursor) = Vec::<Error>::decode_impl(buffer, cursor)?;
                Ok((LogEntry::ResolveAliasEnd { id, has_error, last_errors }, cursor))
            },
            Some(2) => {
                let (n, cursor) = u32::decode_impl(buffer, cursor + 1)?;
                Ok((LogEntry::ResolveAliasLoopStart(n), cursor))
            },
            Some(3) => {
                let (n, cursor) = u32::decode_impl(buffer, cursor + 1)?;
                Ok((LogEntry::ResolveAliasLoopEnd(n), cursor))
            },
            Some(4) => {
                let (id, cursor) = LogId::decode_impl(buffer, cursor + 1)?;
                let (kind, cursor) = ItemKind::decode_impl(buffer, cursor)?;
                let (name, cursor) = Option::<InternedString>::decode_impl(buffer, cursor)?;
                let (span, cursor) = Span::decode_impl(buffer, cursor)?;
                Ok((LogEntry::ResolveItemStart { id, kind, name, span }, cursor))
            },
            Some(5) => {
                let (id, cursor) = LogId::decode_impl(buffer, cursor + 1)?;
                let (has_error, cursor) = bool::decode_impl(buffer, cursor)?;
                let (last_errors, cursor) = Vec::<Error>::decode_impl(buffer, cursor)?;
                Ok((LogEntry::ResolveItemEnd { id, has_error, last_errors }, cursor))
            },
            Some(6) => {
                let (id, cursor) = LogId::decode_impl(buffer, cursor + 1)?;
                let (r#use, cursor) = Use::decode_impl(buffer, cursor)?;
                Ok((LogEntry::ResolveUseStart { id, r#use }, cursor))
            },
            Some(7) => {
                let (id, cursor) = LogId::decode_impl(buffer, cursor + 1)?;
                let (r#use, cursor) = Use::decode_impl(buffer, cursor)?;
                let (has_error, cursor) = bool::decode_impl(buffer, cursor)?;
                let (last_errors, cursor) = Vec::<Error>::decode_impl(buffer, cursor)?;
                Ok((LogEntry::ResolveUseEnd { id, r#use, has_error, last_errors }, cursor))
            },
            Some(8) => {
                let (id, cursor) = LogId::decode_impl(buffer, cursor + 1)?;
                let (path, cursor) = Path::decode_impl(buffer, cursor)?;
                let (type_args, cursor) = Option::<Vec<Type>>::decode_impl(buffer, cursor)?;
                Ok((LogEntry::ResolvePathStart { id, path, type_args }, cursor))
            },
            Some(9) => {
                let (id, cursor) = LogId::decode_impl(buffer, cursor + 1)?;
                let (path, cursor) = Path::decode_impl(buffer, cursor)?;
                let (has_error, cursor) = bool::decode_impl(buffer, cursor)?;
                let (last_errors, cursor) = Vec::<Error>::decode_impl(buffer, cursor)?;
                Ok((LogEntry::ResolvePathEnd { id, path, has_error, last_errors }, cursor))
            },
            Some(10) => {
                let (id, cursor) = LogId::decode_impl(buffer, cursor + 1)?;
                let (expr, cursor) = Expr::decode_impl(buffer, cursor)?;
                Ok((LogEntry::ResolveExprStart { id, expr }, cursor))
            },
            Some(11) => {
                let (id, cursor) = LogId::decode_impl(buffer, cursor + 1)?;
                let (expr, cursor) = Expr::decode_impl(buffer, cursor)?;
                let (has_error, cursor) = bool::decode_impl(buffer, cursor)?;
                let (last_errors, cursor) = Vec::<Error>::decode_impl(buffer, cursor)?;
                Ok((LogEntry::ResolveExprEnd { id, expr, has_error, last_errors }, cursor))
            },
            Some(12) => {
                let (id, cursor) = LogId::decode_impl(buffer, cursor + 1)?;
                let (pattern, cursor) = Pattern::decode_impl(buffer, cursor)?;
                Ok((LogEntry::ResolvePatternStart { id, pattern }, cursor))
            },
            Some(13) => {
                let (id, cursor) = LogId::decode_impl(buffer, cursor + 1)?;
                let (pattern, cursor) = Pattern::decode_impl(buffer, cursor)?;
                let (has_error, cursor) = bool::decode_impl(buffer, cursor)?;
                let (last_errors, cursor) = Vec::<Error>::decode_impl(buffer, cursor)?;
                Ok((LogEntry::ResolvePatternEnd { id, pattern, has_error, last_errors }, cursor))
            },
            Some(14) => {
                let (id, cursor) = LogId::decode_impl(buffer, cursor + 1)?;
                let (r#type, cursor) = Type::decode_impl(buffer, cursor)?;
                Ok((LogEntry::ResolveTypeStart { id, r#type }, cursor))
            },
            Some(15) => {
                let (id, cursor) = LogId::decode_impl(buffer, cursor + 1)?;
                let (r#type, cursor) = Type::decode_impl(buffer, cursor)?;
                let (has_error, cursor) = bool::decode_impl(buffer, cursor)?;
                let (last_errors, cursor) = Vec::<Error>::decode_impl(buffer, cursor)?;
                Ok((LogEntry::ResolveTypeEnd { id, r#type, has_error, last_errors }, cursor))
            },
            Some(n @ 16..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
