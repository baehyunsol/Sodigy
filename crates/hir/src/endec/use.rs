use crate::{Use, Visibility};
use sodigy_endec::{DecodeError, Endec};
use sodigy_name_analysis::IdentWithOrigin;
use sodigy_parse::Field;
use sodigy_span::Span;
use sodigy_string::InternedString;

impl Endec for Use {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.visibility.encode_impl(buffer);
        self.keyword_span.encode_impl(buffer);
        self.name.encode_impl(buffer);
        self.name_span.encode_impl(buffer);
        self.fields.encode_impl(buffer);
        self.root.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (visibility, cursor) = Visibility::decode_impl(buffer, cursor)?;
        let (keyword_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (name, cursor) = InternedString::decode_impl(buffer, cursor)?;
        let (name_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (fields, cursor) = Vec::<Field>::decode_impl(buffer, cursor)?;
        let (root, cursor) = IdentWithOrigin::decode_impl(buffer, cursor)?;

        Ok((
            Use {
                visibility,
                keyword_span,
                name,
                name_span,
                fields,
                root,
            },
            cursor,
        ))
    }
}
