use super::{HirError, HirErrorKind};
use smallvec::SmallVec;
use sodigy_endec::{
    DumpJson,
    Endec,
    EndecError,
    EndecSession,
    JsonObj,
};
use sodigy_error::{ExtraErrorInfo, SodigyError};
use sodigy_intern::InternedString;
use sodigy_span::SpanRange;

impl Endec for HirError {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        self.kind.encode(buffer, session);
        self.spans.encode(buffer, session);
        self.extra.encode(buffer, session);
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(HirError {
            kind: HirErrorKind::decode(buffer, index, session)?,
            spans: SmallVec::<[SpanRange; 1]>::decode(buffer, index, session)?,
            extra: ExtraErrorInfo::decode(buffer, index, session)?,
        })
    }
}

impl Endec for HirErrorKind {
    fn encode(&self, buffer: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            HirErrorKind::NameCollision(name) => {
                buffer.push(0);
                name.encode(buffer, session);
            },
            HirErrorKind::NoDependentTypes(ty) => {
                buffer.push(1);
                ty.encode(buffer, session);
            },
            HirErrorKind::UndefinedName { name, suggestions } => {
                buffer.push(2);
                name.encode(buffer, session);
                suggestions.encode(buffer, session);
            },
            HirErrorKind::UndefinedDeco(deco) => {
                buffer.push(3);
                deco.encode(buffer, session);
            },
            HirErrorKind::RefutablePatternInLet => { buffer.push(4); },
            HirErrorKind::OpenInclusiveRange => { buffer.push(5); },
            HirErrorKind::UnmatchablePattern => { buffer.push(6); },
            HirErrorKind::MultipleShorthands => { buffer.push(7); },
            HirErrorKind::InclusiveStringPattern => { buffer.push(8); },
            HirErrorKind::NameBindingNotAllowedHere => { buffer.push(9); },
            HirErrorKind::TyAnnoNotAllowedHere => { buffer.push(10); },
            HirErrorKind::NameNotBoundInAllPatterns(name) => {
                buffer.push(11);
                name.encode(buffer, session);
            },
            HirErrorKind::TyError => { buffer.push(12); },
            HirErrorKind::TODO(s) => {
                buffer.push(13);
                s.encode(buffer, session);
            },
        }
    }

    fn decode(buffer: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buffer.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(HirErrorKind::NameCollision(InternedString::decode(buffer, index, session)?)),
                    1 => Ok(HirErrorKind::NoDependentTypes(InternedString::decode(buffer, index, session)?)),
                    2 => Ok(HirErrorKind::UndefinedName {
                        name: InternedString::decode(buffer, index, session)?,
                        suggestions: Vec::<InternedString>::decode(buffer, index, session)?,
                    }),
                    3 => Ok(HirErrorKind::UndefinedDeco(InternedString::decode(buffer, index, session)?)),
                    4 => Ok(HirErrorKind::RefutablePatternInLet),
                    5 => Ok(HirErrorKind::OpenInclusiveRange),
                    6 => Ok(HirErrorKind::UnmatchablePattern),
                    7 => Ok(HirErrorKind::MultipleShorthands),
                    8 => Ok(HirErrorKind::InclusiveStringPattern),
                    9 => Ok(HirErrorKind::NameBindingNotAllowedHere),
                    10 => Ok(HirErrorKind::TyAnnoNotAllowedHere),
                    11 => Ok(HirErrorKind::NameNotBoundInAllPatterns(InternedString::decode(buffer, index, session)?)),
                    12 => Ok(HirErrorKind::TyError),
                    13 => Ok(HirErrorKind::TODO(String::decode(buffer, index, session)?)),
                    14.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}

impl DumpJson for HirError {
    fn dump_json(&self) -> JsonObj {
        self.dump_json_impl()
    }
}
