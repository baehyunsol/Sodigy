use crate::{Alias, Generic, Type, Visibility};
use sodigy_endec::{DecodeError, Endec};
use sodigy_name_analysis::NameOrigin;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

impl Endec for Alias {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.visibility.encode_impl(buffer);
        self.keyword_span.encode_impl(buffer);
        self.name.encode_impl(buffer);
        self.name_span.encode_impl(buffer);
        self.generics.encode_impl(buffer);
        self.generic_group_span.encode_impl(buffer);
        self.r#type.encode_impl(buffer);
        self.foreign_names.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (visibility, cursor) = Visibility::decode_impl(buffer, cursor)?;
        let (keyword_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (name, cursor) = InternedString::decode_impl(buffer, cursor)?;
        let (name_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (generics, cursor) = Vec::<Generic>::decode_impl(buffer, cursor)?;
        let (generic_group_span, cursor) = Option::<Span>::decode_impl(buffer, cursor)?;
        let (r#type, cursor) = Type::decode_impl(buffer, cursor)?;
        let (foreign_names, cursor) = HashMap::<InternedString, (NameOrigin, Span)>::decode_impl(buffer, cursor)?;

        Ok((
            Alias {
                visibility,
                keyword_span,
                name,
                name_span,
                generics,
                generic_group_span,
                r#type,
                foreign_names,
            },
            cursor,
        ))
    }
}
