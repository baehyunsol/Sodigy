use crate::{
    ty::Type,
    ty_class::TypeClass,
};
use smallvec::{SmallVec, smallvec};
use sodigy_ast::IdentWithSpan;
use sodigy_error::{
    ExtraErrInfo,
    SodigyError,
    SodigyErrorKind,
};
use sodigy_intern::{InternedString, InternSession};
use sodigy_span::SpanRange;

mod endec;

pub struct MirError {
    kind: MirErrorKind,
    spans: SmallVec<[SpanRange; 1]>,
    extra: ExtraErrInfo,
}

impl MirError {
    pub fn file_not_found(module: IdentWithSpan, path: String) -> Self {
        MirError {
            kind: MirErrorKind::FileNotFound {
                module_name: module.id(),
                path,
            },
            spans: smallvec![*module.span()],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn type_class_not_implemented(
        type_class: TypeClass,
        types: Vec<Type>,  // TODO: use this arg
        span: SpanRange,
    ) -> Self {
        MirError {
            kind: MirErrorKind::TypeClassNotImplemented(type_class),
            spans: smallvec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn type_mismatch(
        expected_ty: Type,
        expected_span: Option<SpanRange>,
        got_ty: Type,
        got_span: SpanRange,
    ) -> Self {
        let mut spans = smallvec![];

        if let Some(sp) = expected_span {
            spans.push(sp);
        }

        spans.push(got_span);

        MirError {
            kind: MirErrorKind::TypeMisMatch {
                expected: expected_ty,
                got: got_ty,
            },
            spans,
            extra: ExtraErrInfo::none(),
        }
    }
}

impl SodigyError<MirErrorKind> for MirError {
    fn get_mut_error_info(&mut self) -> &mut ExtraErrInfo {
        &mut self.extra
    }

    fn get_error_info(&self) -> &ExtraErrInfo {
        &self.extra
    }

    fn get_first_span(&self) -> Option<SpanRange> {
        self.spans.get(0).copied()
    }

    fn get_spans(&self) -> &[SpanRange] {
        &self.spans
    }

    fn error_kind(&self) -> &MirErrorKind {
        &self.kind
    }

    fn index(&self) -> u32 {
        8
    }
}

pub enum MirErrorKind {
    FileNotFound {
        module_name: InternedString,
        path: String,
    },
    TypeClassNotImplemented(TypeClass),
    TypeMisMatch {
        expected: Type,
        got: Type,
    },
}

impl SodigyErrorKind for MirErrorKind {
    fn msg(&self, _: &mut InternSession) -> String {
        todo!()
    }

    fn help(&self, _: &mut InternSession) -> String {
        todo!()
    }

    fn index(&self) -> u32 {
        todo!()
    }
}
