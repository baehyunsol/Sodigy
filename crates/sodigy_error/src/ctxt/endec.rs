use super::ErrorContext;
use sodigy_endec::{Endec, EndecError, EndecSession};

impl Endec for ErrorContext {
    fn encode(&self, buffer: &mut Vec<u8>, _: &mut EndecSession) {
        match self {
            ErrorContext::Unknown => { buffer.push(0); },
            ErrorContext::ParsingCommandLine => { buffer.push(1); },
            ErrorContext::ParsingConfigFile => { buffer.push(2); },
            ErrorContext::ExpandingMacro => { buffer.push(3); },
            ErrorContext::Lexing => { buffer.push(4); },
            ErrorContext::LexingNumericLiteral => { buffer.push(5); },
            ErrorContext::ParsingLetStatement => { buffer.push(6); },
            ErrorContext::ParsingImportStatement => { buffer.push(7); },
            ErrorContext::ParsingFuncName => { buffer.push(8); },
            ErrorContext::ParsingFuncRetType => { buffer.push(9); },
            ErrorContext::ParsingFuncBody => { buffer.push(10); },
            ErrorContext::ParsingFuncArgs => { buffer.push(11); },
            ErrorContext::ParsingEnumBody => { buffer.push(12); },
            ErrorContext::ParsingStructBody => { buffer.push(13); },
            ErrorContext::ParsingStructInit => { buffer.push(14); },
            ErrorContext::ParsingMatchBody => { buffer.push(15); },
            ErrorContext::ParsingBranchCondition => { buffer.push(16); },
            ErrorContext::ParsingLambdaBody => { buffer.push(17); },
            ErrorContext::ParsingScopeBlock => { buffer.push(18); },
            ErrorContext::ParsingFormattedString => { buffer.push(19); },
            ErrorContext::ParsingPattern => { buffer.push(20); },
            ErrorContext::ParsingTypeInPattern => { buffer.push(21); },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, _: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(ErrorContext::Unknown),
                    1 => Ok(ErrorContext::ParsingCommandLine),
                    2 => Ok(ErrorContext::ParsingConfigFile),
                    3 => Ok(ErrorContext::ExpandingMacro),
                    4 => Ok(ErrorContext::Lexing),
                    5 => Ok(ErrorContext::LexingNumericLiteral),
                    6 => Ok(ErrorContext::ParsingLetStatement),
                    7 => Ok(ErrorContext::ParsingImportStatement),
                    8 => Ok(ErrorContext::ParsingFuncName),
                    9 => Ok(ErrorContext::ParsingFuncRetType),
                    10 => Ok(ErrorContext::ParsingFuncBody),
                    11 => Ok(ErrorContext::ParsingFuncArgs),
                    12 => Ok(ErrorContext::ParsingEnumBody),
                    13 => Ok(ErrorContext::ParsingStructBody),
                    14 => Ok(ErrorContext::ParsingStructInit),
                    15 => Ok(ErrorContext::ParsingMatchBody),
                    16 => Ok(ErrorContext::ParsingBranchCondition),
                    17 => Ok(ErrorContext::ParsingLambdaBody),
                    18 => Ok(ErrorContext::ParsingScopeBlock),
                    19 => Ok(ErrorContext::ParsingFormattedString),
                    20 => Ok(ErrorContext::ParsingPattern),
                    21 => Ok(ErrorContext::ParsingTypeInPattern),
                    22.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}
