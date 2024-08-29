use crate::flag::Flag;
use smallvec::{smallvec, SmallVec};
use sodigy_error::{
    ErrorContext,
    ExtraErrInfo,
    RenderError,
    SodigyError,
    SodigyErrorKind,
};
use sodigy_intern::InternSession;
use sodigy_span::SpanRange;

pub struct ClapWarning {
    kind: ClapWarningKind,
    spans: SmallVec<[SpanRange; 1]>,
    extra: ExtraErrInfo,
}

impl ClapWarning {
    pub fn incompatible_flags(
        flag1: Flag,
        span1: SpanRange,
        flag2: Flag,
        span2: SpanRange,
    ) -> Self {
        ClapWarning {
            kind: ClapWarningKind::IncompatibleFlags(flag1, flag2),
            spans: smallvec![span1, span2],
            extra: ExtraErrInfo::at_context(ErrorContext::ParsingCommandLine),
        }
    }

    pub fn unnecessary_flag(flag: Flag, span: SpanRange) -> Self {
        ClapWarning {
            kind: ClapWarningKind::UnnecessaryFlag(flag),
            spans: smallvec![span],
            extra: ExtraErrInfo::at_context(ErrorContext::ParsingCommandLine),
        }
    }
}

impl SodigyError<ClapWarningKind> for ClapWarning {
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

    fn error_kind(&self) -> &ClapWarningKind {
        &self.kind
    }

    fn is_warning(&self) -> bool {
        true
    }

    // do we even need this?
    fn index(&self) -> u32 {
        3
    }
}

pub enum ClapWarningKind {
    IncompatibleFlags(Flag, Flag),
    UnnecessaryFlag(Flag),
}

impl SodigyErrorKind for ClapWarningKind {
    fn msg(&self, _: &mut InternSession) -> String {
        match self {
            ClapWarningKind::IncompatibleFlags(flag1, flag2) => format!("`{}` and `{}` are incompatible", flag1.render_error(), flag2.render_error()),
            ClapWarningKind::UnnecessaryFlag(flag) => format!("`{}` doesn't do anything", flag.render_error()),
        }
    }

    fn help(&self, _: &mut InternSession) -> String {
        match self {
            ClapWarningKind::IncompatibleFlags(flag1, flag2) => match (flag1, flag2) {
                (Flag::Hir, Flag::DumpMirTo)
                | (Flag::DumpMirTo, Flag::Hir) => format!(
                    "`{}` does not generate mir, so there's not mir to dump!",
                    Flag::Hir.render_error(),
                ),
                (Flag::Hir, Flag::Library)
                | (Flag::Library, Flag::Hir) => format!(
                    "`{}` stops the compilation at the hir pass, and it doesn't have to look for libraries when generating hir.",
                    Flag::Hir.render_error(),
                ),
                _ => String::new(),
            },
            ClapWarningKind::UnnecessaryFlag(flag) => match flag {
                Flag::DumpType => format!(
                    "`{}` does nothing without `{}` or `{}`. If there's nothing to dump, what 'dump' type does it set?",
                    Flag::DumpType.render_error(),
                    Flag::DumpHirTo.render_error(),
                    Flag::DumpMirTo.render_error(),
                ),
                _ => String::new(),
            },
        }
    }

    // we don't need this, but I want it to look more complete
    fn index(&self) -> u32 {
        match self {
            ClapWarningKind::IncompatibleFlags(_, _) => 0,
            ClapWarningKind::UnnecessaryFlag(_) => 1,
        }
    }
}
