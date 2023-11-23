use super::FormattedStringElement;
use crate::TokenTree;
use sodigy_endec::{Endec, EndecErr, EndecSession};

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

    fn decode(buf: &[u8], ind: &mut usize, session: &mut EndecSession) -> Result<Self, EndecErr> {
        match buf.get(*ind) {
            Some(n) => {
                *ind += 1;

                match *n {
                    0 => Ok(FormattedStringElement::Value(Vec::<TokenTree>::decode(buf, ind, session)?)),
                    1 => Ok(FormattedStringElement::Literal(String::decode(buf, ind, session)?)),
                    2.. => Err(EndecErr::InvalidEnumVariant { variant_index: *n }),
                }
            },
            None => Err(EndecErr::Eof),
        }
    }
}
