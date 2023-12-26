use super::ExpectedToken;
use sodigy_endec::{Endec, EndecError, EndecSession};

impl<T: Endec> Endec for ExpectedToken<T> {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            ExpectedToken::AnyIdentifier => { buf.push(0); },
            ExpectedToken::AnyExpression => { buf.push(1); },
            ExpectedToken::AnyStatement => { buf.push(2); },
            ExpectedToken::AnyDocComment => { buf.push(3); },
            ExpectedToken::AnyPattern => { buf.push(4); },
            ExpectedToken::AnyType => { buf.push(5); },
            ExpectedToken::AnyNumber => { buf.push(6); },
            ExpectedToken::IdentOrBrace => { buf.push(7); },
            ExpectedToken::Nothing => { buf.push(8); },
            ExpectedToken::PostExpr => { buf.push(9); },
            ExpectedToken::FuncArgs => { buf.push(10); },
            ExpectedToken::Specific(tokens) => {
                buf.push(11);
                tokens.encode(buf, session);
            },
            ExpectedToken::LetStatement => { buf.push(12); },
        }
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buf.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(ExpectedToken::AnyIdentifier),
                    1 => Ok(ExpectedToken::AnyExpression),
                    2 => Ok(ExpectedToken::AnyStatement),
                    3 => Ok(ExpectedToken::AnyDocComment),
                    4 => Ok(ExpectedToken::AnyPattern),
                    5 => Ok(ExpectedToken::AnyType),
                    6 => Ok(ExpectedToken::AnyNumber),
                    7 => Ok(ExpectedToken::IdentOrBrace),
                    8 => Ok(ExpectedToken::Nothing),
                    9 => Ok(ExpectedToken::PostExpr),
                    10 => Ok(ExpectedToken::FuncArgs),
                    11 => Ok(ExpectedToken::Specific(Vec::<T>::decode(buf, index, session)?)),
                    12 => Ok(ExpectedToken::LetStatement),
                    13.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}
