use crate::{Enum, EnumFieldKind};
use sodigy_endec::{DecodeError, Endec};
use sodigy_hir::Generic;
use sodigy_span::Span;
use sodigy_string::InternedString;

impl Endec for Enum {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.name.encode_impl(buffer);
        self.name_span.encode_impl(buffer);
        self.generics.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (name, cursor) = InternedString::decode_impl(buffer, cursor)?;
        let (name_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (generics, cursor) = Vec::<Generic>::decode_impl(buffer, cursor)?;

        Ok((
            Enum {
                name,
                name_span,
                generics,
            },
            cursor,
        ))
    }
}

impl Endec for EnumFieldKind {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            EnumFieldKind::None => {
                buffer.push(0);
            },
            EnumFieldKind::Tuple => {
                buffer.push(1);
            },
            EnumFieldKind::Struct => {
                buffer.push(2);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((EnumFieldKind::None, cursor + 1)),
            Some(1) => Ok((EnumFieldKind::Tuple, cursor + 1)),
            Some(2) => Ok((EnumFieldKind::Struct, cursor + 1)),
            Some(n @ 3..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
