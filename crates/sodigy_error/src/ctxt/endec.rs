use super::ErrorContext;
use sodigy_endec::{Endec, EndecError, EndecSession};

impl Endec for ErrorContext {
    fn encode(&self, buffer: &mut Vec<u8>, _: &mut EndecSession) {
        match self {
            ErrorContext::Unknown => { buffer.push(0); },
            ErrorContext::ParsingCommandLine => { buffer.push(1); },
            ErrorContext::ExpandingMacro => { buffer.push(2); },
            ErrorContext::Lexing => { buffer.push(3); },
            ErrorContext::LexingNumericLiteral => { buffer.push(4); },
            ErrorContext::ParsingLetStatement => { buffer.push(5); },
            ErrorContext::ParsingImportStatement => { buffer.push(6); },
            ErrorContext::ParsingFuncName => { buffer.push(7); },
            ErrorContext::ParsingFuncRetType => { buffer.push(8); },
            ErrorContext::ParsingFuncBody => { buffer.push(9); },
            ErrorContext::ParsingFuncArgs => { buffer.push(10); },
            ErrorContext::ParsingEnumBody => { buffer.push(11); },
            ErrorContext::ParsingStructBody => { buffer.push(12); },
            ErrorContext::ParsingStructInit => { buffer.push(13); },
            ErrorContext::ParsingMatchBody => { buffer.push(14); },
            ErrorContext::ParsingBranchCondition => { buffer.push(15); },
            ErrorContext::ParsingLambdaBody => { buffer.push(16); },
            ErrorContext::ParsingScopeBlock => { buffer.push(17); },
            ErrorContext::ParsingFormattedString => { buffer.push(18); },
            ErrorContext::ParsingPattern => { buffer.push(19); },
            ErrorContext::ParsingTypeInPattern => { buffer.push(20); },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, _: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
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
