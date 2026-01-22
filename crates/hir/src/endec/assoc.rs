use crate::{AssociatedItem, AssociatedItemKind, Type};
use sodigy_endec::{DecodeError, Endec};
use sodigy_span::Span;
use sodigy_string::InternedString;

impl Endec for AssociatedItem {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.kind.encode_impl(buffer);
        self.name.encode_impl(buffer);
        self.name_span.encode_impl(buffer);
        self.is_pure.encode_impl(buffer);
        self.params.encode_impl(buffer);
        self.type_span.encode_impl(buffer);
        self.r#type.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (kind, cursor) = AssociatedItemKind::decode_impl(buffer, cursor)?;
        let (name, cursor) = InternedString::decode_impl(buffer, cursor)?;
        let (name_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (is_pure, cursor) = Option::<bool>::decode_impl(buffer, cursor)?;
        let (params, cursor) = Option::<usize>::decode_impl(buffer, cursor)?;
        let (type_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (r#type, cursor) = Type::decode_impl(buffer, cursor)?;

        Ok((
            AssociatedItem {
                kind,
                name,
                name_span,
                is_pure,
                params,
                type_span,
                r#type,
            },
            cursor,
        ))
    }
}

impl Endec for AssociatedItemKind {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            AssociatedItemKind::Func => {
                buffer.push(0);
            },
            AssociatedItemKind::Let => {
                buffer.push(1);
            },
            AssociatedItemKind::Field => {
                buffer.push(2);
            },
            AssociatedItemKind::Variant => {
                buffer.push(3);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => Ok((AssociatedItemKind::Func, cursor + 1)),
            Some(1) => Ok((AssociatedItemKind::Let, cursor + 1)),
            Some(2) => Ok((AssociatedItemKind::Field, cursor + 1)),
            Some(3) => Ok((AssociatedItemKind::Variant, cursor + 1)),
            Some(n @ 4..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}
