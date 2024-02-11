use super::ExpectedToken;
use sodigy_endec::{Endec, EndecError, EndecSession};

impl<T: Endec> Endec for ExpectedToken<T> {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            ExpectedToken::AnyIdentifier => { buffer.push(0); },
            ExpectedToken::AnyExpression => { buffer.push(1); },
            ExpectedToken::AnyStatement => { buffer.push(2); },
            ExpectedToken::AnyDocComment => { buffer.push(3); },
            ExpectedToken::AnyPattern => { buffer.push(4); },
            ExpectedToken::AnyType => { buffer.push(5); },
            ExpectedToken::AnyNumber => { buffer.push(6); },
            ExpectedToken::IdentOrBrace => { buffer.push(7); },
            ExpectedToken::Nothing => { buffer.push(8); },
            ExpectedToken::PostExpr => { buffer.push(9); },
            ExpectedToken::FuncArgs => { buffer.push(10); },
            ExpectedToken::Specific(tokens) => {
                buffer.push(11);
                tokens.encode(buffer, session);
            },
            ExpectedToken::LetStatement => { buffer.push(12); },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
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
                    11 => Ok(ExpectedToken::Specific(Vec::<T>::decode(buffer, index, session)?)),
                    12 => Ok(ExpectedToken::LetStatement),
                    13.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}
