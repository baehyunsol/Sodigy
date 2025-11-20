use crate::{Counter, IdentWithOrigin, NameKind, NameOrigin, UseCount};
use sodigy_endec::{DecodeError, Endec};
use sodigy_span::Span;
use sodigy_string::InternedString;

impl Endec for IdentWithOrigin {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.id.encode_impl(buffer);
        self.span.encode_impl(buffer);
        self.origin.encode_impl(buffer);
        self.def_span.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (id, cursor) = InternedString::decode_impl(buffer, cursor)?;
        let (span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (origin, cursor) = NameOrigin::decode_impl(buffer, cursor)?;
        let (def_span, cursor) = Span::decode_impl(buffer, cursor)?;

        Ok((
            IdentWithOrigin {
                id,
                span,
                origin,
                def_span,
            },
            cursor,
        ))
    }
}

impl Endec for NameOrigin {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            NameOrigin::FuncParam { index } => {
                buffer.push(0);
                index.encode_impl(buffer);
            },
            NameOrigin::Generic { index } => {
                buffer.push(1);
                index.encode_impl(buffer);
            },
            NameOrigin::Local { kind } => {
                buffer.push(2);
                kind.encode_impl(buffer);
            },
            NameOrigin::Foreign { kind } => {
                buffer.push(3);
                kind.encode_impl(buffer);
            },
            NameOrigin::External => {
                buffer.push(4);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (index, cursor) = usize::decode_impl(buffer, cursor + 1)?;
                Ok((NameOrigin::FuncParam { index }, cursor))
            },
            Some(1) => {
                let (index, cursor) = usize::decode_impl(buffer, cursor + 1)?;
                Ok((NameOrigin::Generic { index }, cursor))
            },
            Some(2) => {
                let (kind, cursor) = NameKind::decode_impl(buffer, cursor + 1)?;
                Ok((NameOrigin::Local { kind }, cursor))
            },
            Some(3) => {
                let (kind, cursor) = NameKind::decode_impl(buffer, cursor + 1)?;
                Ok((NameOrigin::Foreign { kind }, cursor))
            },
            Some(4) => Ok((NameOrigin::External, cursor + 1)),
            Some(n @ 5..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for NameKind {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            NameKind::Let { is_top_level: true } => {
                buffer.push(0);
            },
            NameKind::Let { is_top_level: false } => {
                buffer.push(1);
            },
            NameKind::Func => {
                buffer.push(2);
            },
            NameKind::Struct => {
                buffer.push(3);
            },
            NameKind::Enum => {
                buffer.push(4);
            },
            NameKind::EnumVariant { parent } => {
                buffer.push(5);
                parent.encode_impl(buffer);
            },
            NameKind::Alias => {
                buffer.push(6);
            },
            NameKind::Module => {
                buffer.push(7);
            },
            NameKind::Use => {
                buffer.push(8);
            },
            NameKind::FuncParam => {
                buffer.push(9);
            },
            NameKind::Generic => {
                buffer.push(10);
            },
            NameKind::PatternNameBind => {
                buffer.push(11);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((NameKind::Let { is_top_level: true }, cursor + 1)),
            Some(1) => Ok((NameKind::Let { is_top_level: false }, cursor + 1)),
            Some(2) => Ok((NameKind::Func, cursor + 1)),
            Some(3) => Ok((NameKind::Struct, cursor + 1)),
            Some(4) => Ok((NameKind::Enum, cursor + 1)),
            Some(5) => {
                let (parent, cursor) = Span::decode_impl(buffer, cursor + 1)?;
                Ok((NameKind::EnumVariant { parent }, cursor))
            },
            Some(6) => Ok((NameKind::Alias, cursor + 1)),
            Some(7) => Ok((NameKind::Module, cursor + 1)),
            Some(8) => Ok((NameKind::Use, cursor + 1)),
            Some(9) => Ok((NameKind::FuncParam, cursor + 1)),
            Some(10) => Ok((NameKind::Generic, cursor + 1)),
            Some(11) => Ok((NameKind::PatternNameBind, cursor + 1)),
            Some(n @ 12..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for UseCount {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            UseCount {
                always: Counter::Never,
                debug_only: Counter::Never,
            } => {
                buffer.push(0);
            },
            UseCount {
                always: Counter::Never,
                debug_only: Counter::Once,
            } => {
                buffer.push(1);
            },
            UseCount {
                always: Counter::Never,
                debug_only: Counter::Multiple,
            } => {
                buffer.push(2);
            },
            UseCount {
                always: Counter::Once,
                debug_only: Counter::Never,
            } => {
                buffer.push(3);
            },
            UseCount {
                always: Counter::Once,
                debug_only: Counter::Once,
            } => {
                buffer.push(4);
            },
            UseCount {
                always: Counter::Once,
                debug_only: Counter::Multiple,
            } => {
                buffer.push(5);
            },
            UseCount {
                always: Counter::Multiple,
                debug_only: Counter::Never,
            } => {
                buffer.push(6);
            },
            UseCount {
                always: Counter::Multiple,
                debug_only: Counter::Once,
            } => {
                buffer.push(7);
            },
            UseCount {
                always: Counter::Multiple,
                debug_only: Counter::Multiple,
            } => {
                buffer.push(8);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((
                UseCount {
                    always: Counter::Never,
                    debug_only: Counter::Never,
                },
                cursor + 1,
            )),
            Some(1) => Ok((
                UseCount {
                    always: Counter::Never,
                    debug_only: Counter::Once,
                },
                cursor + 1,
            )),
            Some(2) => Ok((
                UseCount {
                    always: Counter::Never,
                    debug_only: Counter::Multiple,
                },
                cursor + 1,
            )),
            Some(3) => Ok((
                UseCount {
                    always: Counter::Once,
                    debug_only: Counter::Never,
                },
                cursor + 1,
            )),
            Some(4) => Ok((
                UseCount {
                    always: Counter::Once,
                    debug_only: Counter::Once,
                },
                cursor + 1,
            )),
            Some(5) => Ok((
                UseCount {
                    always: Counter::Once,
                    debug_only: Counter::Multiple,
                },
                cursor + 1,
            )),
            Some(6) => Ok((
                UseCount {
                    always: Counter::Multiple,
                    debug_only: Counter::Never,
                },
                cursor + 1,
            )),
            Some(7) => Ok((
                UseCount {
                    always: Counter::Multiple,
                    debug_only: Counter::Once,
                },
                cursor + 1,
            )),
            Some(8) => Ok((
                UseCount {
                    always: Counter::Multiple,
                    debug_only: Counter::Multiple,
                },
                cursor + 1,
            )),
            Some(n @ 9..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
