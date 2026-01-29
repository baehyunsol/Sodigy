use crate::{Path, Use, Visibility};
use sodigy_endec::{DecodeError, Endec};
use sodigy_span::Span;
use sodigy_string::InternedString;

impl Endec for Use {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.visibility.encode_impl(buffer);
        self.keyword_span.encode_impl(buffer);
        self.name.encode_impl(buffer);
        self.name_span.encode_impl(buffer);
        self.path.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (visibility, cursor) = Visibility::decode_impl(buffer, cursor)?;
        let (keyword_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (name, cursor) = InternedString::decode_impl(buffer, cursor)?;
        let (name_span, cursor) = Span::decode_impl(buffer, cursor)?;
        let (path, cursor) = Path::decode_impl(buffer, cursor)?;

        Ok((
            Use {
                visibility,
                keyword_span,
                name,
                name_span,
                path,
            },
            cursor,
        ))
    }
}
