use super::FormattedStringElement;
use crate::TokenTree;
use sodigy_endec::{Endec, EndecError, EndecSession};

/*pub enum FormattedStringElement {
    Value(Vec<TokenTree>),
    Literal(String),
}
*/
impl Endec for FormattedStringElement {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            FormattedStringElement::Value(v) => {
                buffer.push(0);
                v.encode(buffer, session);
            },
            FormattedStringElement::Literal(s) => {
                buffer.push(1);
                s.encode(buffer, session);
            },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(FormattedStringElement::Value(Vec::<TokenTree>::decode(buffer, index, session)?)),
                    1 => Ok(FormattedStringElement::Literal(String::decode(buffer, index, session)?)),
                    2.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}
