use crate::{Public, StdAttribute};
use sodigy_endec::{DecodeError, Endec};

impl Endec for Public {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.0.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (public, cursor) = bool::decode_impl(buffer, cursor)?;
        Ok((Public(public), cursor))
    }
}

impl Endec for StdAttribute {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.built_in.encode_impl(buffer);
        self.no_type.encode_impl(buffer);
        self.lang_item.encode_impl(buffer);
        self.lang_item_generics.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (built_in, cursor) = bool::decode_impl(buffer, cursor)?;
        let (no_type, cursor) = bool::decode_impl(buffer, cursor)?;
        let (lang_item, cursor) = Option::<String>::decode_impl(buffer, cursor)?;
        let (lang_item_generics, cursor) = Option::<Vec<String>>::decode_impl(buffer, cursor)?;

        Ok((
            StdAttribute {
                built_in,
                no_type,
                lang_item,
                lang_item_generics,
            },
            cursor,
        ))
    }
}
