use super::ErrorContext;
use sodigy_endec::{Endec, EndecError, EndecSession};

impl Endec for ErrorContext {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            ErrorContext::Unknown => { buf.push(0); },
            ErrorContext::ParsingCommandLine => { buf.push(1); },
            ErrorContext::ExpandingMacro => { buf.push(2); },
            ErrorContext::Lexing => { buf.push(3); },
            ErrorContext::LexingNumericLiteral => { buf.push(4); },
            ErrorContext::ParsingLetStatement => { buf.push(5); },
            ErrorContext::ParsingImportStatement => { buf.push(6); },
            ErrorContext::ParsingFuncName => { buf.push(7); },
            ErrorContext::ParsingFuncRetType => { buf.push(8); },
            ErrorContext::ParsingFuncBody => { buf.push(9); },
            ErrorContext::ParsingFuncArgs => { buf.push(10); },
            ErrorContext::ParsingEnumBody => { buf.push(11); },
            ErrorContext::ParsingStructBody => { buf.push(12); },
            ErrorContext::ParsingStructInit => { buf.push(13); },
            ErrorContext::ParsingMatchBody => { buf.push(14); },
            ErrorContext::ParsingBranchCondition => { buf.push(15); },
            ErrorContext::ParsingLambdaBody => { buf.push(16); },
            ErrorContext::ParsingScopeBlock => { buf.push(17); },
            ErrorContext::ParsingFormattedString => { buf.push(18); },
            ErrorContext::ParsingPattern => { buf.push(19); },
            ErrorContext::ParsingTypeInPattern => { buf.push(20); },
        }
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buf.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(ErrorContext::Unknown),
                    1 => Ok(ErrorContext::ParsingCommandLine),
                    2 => Ok(ErrorContext::ExpandingMacro),
                    3 => Ok(ErrorContext::Lexing),
                    4 => Ok(ErrorContext::LexingNumericLiteral),
                    5 => Ok(ErrorContext::ParsingLetStatement),
                    6 => Ok(ErrorContext::ParsingImportStatement),
                    7 => Ok(ErrorContext::ParsingFuncName),
                    8 => Ok(ErrorContext::ParsingFuncRetType),
                    9 => Ok(ErrorContext::ParsingFuncBody),
                    10 => Ok(ErrorContext::ParsingFuncArgs),
                    11 => Ok(ErrorContext::ParsingEnumBody),
                    12 => Ok(ErrorContext::ParsingStructBody),
                    13 => Ok(ErrorContext::ParsingStructInit),
                    14 => Ok(ErrorContext::ParsingMatchBody),
                    15 => Ok(ErrorContext::ParsingBranchCondition),
                    16 => Ok(ErrorContext::ParsingLambdaBody),
                    17 => Ok(ErrorContext::ParsingScopeBlock),
                    18 => Ok(ErrorContext::ParsingFormattedString),
                    19 => Ok(ErrorContext::ParsingPattern),
                    20 => Ok(ErrorContext::ParsingTypeInPattern),
                    21.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}
