// modules that cannot depend on `sodigy_endec`

use crate::{
    DumpJson,
    Endec,
    EndecError,
    EndecSession,
    JsonObj,
};
use crate::session::EncodedInternal;
use hmath::{BigInt, Ratio};
use sodigy_config::{
    CompilerOption,
    CompilerOutputFormat,
    DumpType,
    SpecialOutput,
};
use sodigy_intern::{InternedString, InternedNumeric};
use sodigy_keyword::Keyword;
use sodigy_number::SodigyNumber;
use std::collections::HashMap;

impl Endec for CompilerOption {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.do_not_compile_and_do_this.encode(buffer, session);
        self.input_path.encode(buffer, session);
        self.output_path.encode(buffer, session);
        self.output_format.encode(buffer, session);
        self.show_warnings.encode(buffer, session);
        self.dump_hir_to.encode(buffer, session);
        self.dump_mir_to.encode(buffer, session);
        self.dump_type.encode(buffer, session);
        self.library_paths.encode(buffer, session);
        self.verbosity.encode(buffer, session);
        self.or_pattern_expansion_limit.encode(buffer, session);
        self.raw_input.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(CompilerOption {
            do_not_compile_and_do_this: Option::<SpecialOutput>::decode(buffer, index, session)?,
            input_path: Option::<String>::decode(buffer, index, session)?,
            output_path: Option::<String>::decode(buffer, index, session)?,
            output_format: CompilerOutputFormat::decode(buffer, index, session)?,
            show_warnings: bool::decode(buffer, index, session)?,
            dump_hir_to: Option::<String>::decode(buffer, index, session)?,
            dump_mir_to: Option::<String>::decode(buffer, index, session)?,
            dump_type: DumpType::decode(buffer, index, session)?,
            library_paths: Option::<HashMap<String, String>>::decode(buffer, index, session)?,
            verbosity: u8::decode(buffer, index, session)?,
            or_pattern_expansion_limit: usize::decode(buffer, index, session)?,
            raw_input: Option::<Vec<u8>>::decode(buffer, index, session)?,
        })
    }
}

impl Endec for SpecialOutput {
    fn encode(&self, buffer: &mut Vec<u8>, _: &mut EndecSession) {
        match self {
            SpecialOutput::HelpMessage => { buffer.push(0); },
            SpecialOutput::VersionInfo => { buffer.push(1); },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, _: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(SpecialOutput::HelpMessage),
                    1 => Ok(SpecialOutput::VersionInfo),
                    2.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}

impl Endec for CompilerOutputFormat {
    fn encode(&self, buffer: &mut Vec<u8>, _: &mut EndecSession) {
        match self {
            CompilerOutputFormat::None => { buffer.push(0); },
            CompilerOutputFormat::Hir => { buffer.push(1); },
            CompilerOutputFormat::Mir => { buffer.push(2); },
            CompilerOutputFormat::Binary => { buffer.push(3); },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, _: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(CompilerOutputFormat::None),
                    1 => Ok(CompilerOutputFormat::Hir),
                    2 => Ok(CompilerOutputFormat::Mir),
                    3 => Ok(CompilerOutputFormat::Binary),
                    4.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}

impl Endec for DumpType {
    fn encode(&self, buffer: &mut Vec<u8>, _: &mut EndecSession) {
        match self {
            DumpType::Json => { buffer.push(0); },
            DumpType::String => { buffer.push(1); },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, _: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(DumpType::Json),
                    1 => Ok(DumpType::String),
                    2.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}

impl Endec for InternedString {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        // TODO: optimization: if this InternedString appears only once, don't intern it: just encode the raw string!
        let e = session.encode_intern_str(*self);
        e.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        let e = EncodedInternal::decode(buffer, index, session)?;
        Ok(session.decode_intern_str(e)?)
    }
}

impl Endec for InternedNumeric {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        // TODO: optimization: if this InternedNumeric appears only once, don't intern it: just encode the raw SodigyNumber!
        let e = session.encode_intern_num(*self);
        e.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        let e = EncodedInternal::decode(buffer, index, session)?;
        Ok(session.decode_intern_num(e)?)
    }
}

impl Endec for Keyword {
    fn encode(&self, buffer: &mut Vec<u8>, _: &mut EndecSession) {
        match self {
            Keyword::Let => { buffer.push(0); },
            Keyword::Enum => { buffer.push(1); },
            Keyword::Struct => { buffer.push(2); },
            Keyword::Module => { buffer.push(3); },
            Keyword::Import => { buffer.push(4); },
            Keyword::As => { buffer.push(5); },
            Keyword::From => { buffer.push(6); },
            Keyword::If => { buffer.push(7); },
            Keyword::Else => { buffer.push(8); },
            Keyword::Pattern => { buffer.push(9); },
            Keyword::Match => { buffer.push(10); },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, _: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(Keyword::Let),
                    1 => Ok(Keyword::Enum),
                    2 => Ok(Keyword::Struct),
                    3 => Ok(Keyword::Module),
                    4 => Ok(Keyword::Import),
                    5 => Ok(Keyword::As),
                    6 => Ok(Keyword::From),
                    7 => Ok(Keyword::If),
                    8 => Ok(Keyword::Else),
                    9 => Ok(Keyword::Pattern),
                    10 => Ok(Keyword::Match),
                    11.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}

impl Endec for SodigyNumber {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            SodigyNumber::BigInt(n) => {
                buffer.push(0);
                n.encode(buffer, session);
            },
            SodigyNumber::BigRatio(n) => {
                buffer.push(1);
                n.encode(buffer, session);
            },
            SodigyNumber::SmallInt(n) => {
                buffer.push(2);
                n.encode(buffer, session);
            },
            SodigyNumber::SmallRatio { denom, numer } => {
                buffer.push(3);
                denom.encode(buffer, session);
                numer.encode(buffer, session);
            },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(SodigyNumber::BigInt(Box::new(BigInt::decode(buffer, index, session)?))),
                    1 => Ok(SodigyNumber::BigRatio(Box::new(Ratio::decode(buffer, index, session)?))),
                    2 => Ok(SodigyNumber::SmallInt(i64::decode(buffer, index, session)?)),
                    3 => Ok(SodigyNumber::SmallRatio {
                        denom: u32::decode(buffer, index, session)?,
                        numer: i32::decode(buffer, index, session)?,
                    }),
                    4.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}

impl Endec for BigInt {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        let (raw, sign) = self.clone().into_raw();

        raw.encode(buffer, session);
        sign.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(BigInt::from_raw(
            Vec::<u32>::decode(buffer, index, session)?,
            bool::decode(buffer, index, session)?,
        ))
    }
}

impl Endec for Ratio {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        let (denom, denom_neg, numer, numer_neg) = self.clone().into_raw();

        denom.encode(buffer, session);
        denom_neg.encode(buffer, session);
        numer.encode(buffer, session);
        numer_neg.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(Ratio::from_raw(
            Vec::<u32>::decode(buffer, index, session)?,
            bool::decode(buffer, index, session)?,
            Vec::<u32>::decode(buffer, index, session)?,
            bool::decode(buffer, index, session)?,
        ))
    }
}

impl DumpJson for InternedString {
    fn dump_json(&self) -> JsonObj {
        self.to_string().dump_json()
    }
}

impl DumpJson for InternedNumeric {
    fn dump_json(&self) -> JsonObj {
        self.to_string().dump_json()
    }
}
