use super::{HirError, HirErrorKind};
use smallvec::SmallVec;
use sodigy_endec::{Endec, EndecError, EndecSession};
use sodigy_error::ExtraErrInfo;
use sodigy_intern::InternedString;
use sodigy_span::SpanRange;

impl Endec for HirError {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        self.kind.encode(buf, session);
        self.spans.encode(buf, session);
        self.extra.encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(HirError {
            kind: HirErrorKind::decode(buf, index, session)?,
            spans: SmallVec::<[SpanRange; 1]>::decode(buf, index, session)?,
            extra: ExtraErrInfo::decode(buf, index, session)?,
        })
    }
}

impl Endec for HirErrorKind {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        match self {
            HirErrorKind::NameCollision(name) => {
                buf.push(0);
                name.encode(buf, session);
            },
            HirErrorKind::NoDependentTypes(ty) => {
                buf.push(1);
                ty.encode(buf, session);
            },
            HirErrorKind::UndefinedName { name, suggestions } => {
                buf.push(2);
                name.encode(buf, session);
                suggestions.encode(buf, session);
            },
            HirErrorKind::UndefinedDeco(deco) => {
                buf.push(3);
                deco.encode(buf, session);
            },
            HirErrorKind::RefutablePatternInLet => { buf.push(4); },
            HirErrorKind::OpenInclusiveRange => { buf.push(5); },
            HirErrorKind::UnmatchablePattern => { buf.push(6); },
            HirErrorKind::MultipleShorthands => { buf.push(7); },
            HirErrorKind::TyError => { buf.push(8); },
            HirErrorKind::TODO(s) => {
                buf.push(9);
                s.encode(buf, session);
            },
        }
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buf.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(HirErrorKind::NameCollision(InternedString::decode(buf, index, session)?)),
                    1 => Ok(HirErrorKind::NoDependentTypes(InternedString::decode(buf, index, session)?)),
                    2 => Ok(HirErrorKind::UndefinedName {
                        name: InternedString::decode(buf, index, session)?,
                        suggestions: Vec::<InternedString>::decode(buf, index, session)?,
                    }),
                    3 => Ok(HirErrorKind::UndefinedDeco(InternedString::decode(buf, index, session)?)),
                    4 => Ok(HirErrorKind::RefutablePatternInLet),
                    5 => Ok(HirErrorKind::OpenInclusiveRange),
                    6 => Ok(HirErrorKind::UnmatchablePattern),
                    7 => Ok(HirErrorKind::MultipleShorthands),
                    8 => Ok(HirErrorKind::TyError),
                    9 => Ok(HirErrorKind::TODO(String::decode(buf, index, session)?)),
                    10.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}
