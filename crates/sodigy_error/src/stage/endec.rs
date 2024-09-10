use super::Stage;
use sodigy_endec::{Endec, EndecError, EndecSession};

impl Endec for Stage {
    fn encode(&self, buffer: &mut Vec<u8>, _: &mut EndecSession) {
        match self {
            Stage::FileIo => { buffer.push(0); },
            Stage::Endec => { buffer.push(1); },
            Stage::Clap => { buffer.push(2); },
            Stage::Lex => { buffer.push(3); },
            Stage::Parse => { buffer.push(4); },
            Stage::Ast => { buffer.push(5); },
            Stage::Hir => { buffer.push(6); },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, _: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(Stage::FileIo),
                    1 => Ok(Stage::Endec),
                    2 => Ok(Stage::Clap),
                    3 => Ok(Stage::Lex),
                    4 => Ok(Stage::Parse),
                    5 => Ok(Stage::Ast),
                    6 => Ok(Stage::Hir),
                    7.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}
