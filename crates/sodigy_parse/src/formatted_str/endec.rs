use super::FormattedStringElement;
use crate::TokenTree;
use sodigy_endec::{Endec, EndecError, EndecSession};

/*pub enum FormattedStringElement {
    Value(Vec<TokenTree>),
    Literal(String),
}
*/
impl Endec for FormattedStringElement {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            FormattedStringElement::Value(v) => {
                buf.push(0);
                v.encode(buf, session);
            },
            FormattedStringElement::Literal(s) => {
                buf.push(1);
                s.encode(buf, session);
            },
        }
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buf.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(FormattedStringElement::Value(Vec::<TokenTree>::decode(buf, index, session)?)),
                    1 => Ok(FormattedStringElement::Literal(String::decode(buf, index, session)?)),
                    2.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}
