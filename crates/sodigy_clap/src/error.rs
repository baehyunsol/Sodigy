use crate::flag::{Flag, FLAGS};
use crate::token::TokenKind;
use hmath::BigInt;
use smallvec::{smallvec, SmallVec};
use sodigy_error::{
    ErrorContext,
    ExtraErrInfo,
    RenderError,
    SodigyError,
    SodigyErrorKind,
    Stage,
    substr_edit_distance,
    trim_long_string,
};
use sodigy_intern::InternSession;
use sodigy_span::SpanRange;

pub struct ClapError {
    kind: ClapErrorKind,
    spans: SmallVec<[SpanRange; 1]>,
    extra: ExtraErrInfo,
}

impl ClapError {
    pub fn invalid_utf8(span: SpanRange) -> Self {
        ClapError {
            kind: ClapErrorKind::InvalidUtf8,
            spans: smallvec![span],
            extra: ExtraErrInfo::at_context(ErrorContext::ParsingCommandLine),
        }
    }

    pub fn invalid_argument(kind: TokenKind, argument: &[u8], span: SpanRange) -> Self {
        match kind {
            TokenKind::Path if argument.get(0) == Some(&b'-') => match String::from_utf8(argument.to_vec()) {
                Ok(s) => {
                    // it catches the typo (this search is very expensive)
                    let mut closest_flag = vec![];
                    let mut closest_dist = usize::MAX;

                    for flag in FLAGS.iter() {
                        if let Some(flag) = flag.long() {
                            let dist = substr_edit_distance(&argument, flag);

                            if dist < closest_dist {
                                closest_dist = dist;
                                closest_flag = flag.to_vec();
                            }
                        }

                        if let Some(flag) = flag.short() {
                            let dist = substr_edit_distance(&argument, flag);

                            if dist < closest_dist {
                                closest_dist = dist;
                                closest_flag = flag.to_vec();
                            }
                        }
                    }

                    let mut extra = ExtraErrInfo::at_context(ErrorContext::ParsingCommandLine);

                    //  --xx -> --to  (no sense)
                    //  --tx -> --to  (makes sense)
                    //  --verrrion -> --version (makes sense)
                    if (argument.len() > 4 && closest_dist < 3) || closest_dist < 2 {
                        extra.set_message(format!("Do you mean `{}`?", String::from_utf8(closest_flag).unwrap()));
                    }

                    ClapError {
                        kind: ClapErrorKind::InvalidArgument(kind, s),
                        spans: smallvec![span],
                        extra,
                    }
                },
                Err(_) => ClapError::invalid_utf8(span),
            },
            _ => ClapError {
                kind: ClapErrorKind::InvalidArgument(kind, String::from_utf8_lossy(argument).to_string()),
                spans: smallvec![span],
                extra: ExtraErrInfo::at_context(ErrorContext::ParsingCommandLine),
            },
        }
    }

    pub fn no_args_at_all() -> Self {
        ClapError {
            kind: ClapErrorKind::NoArgsAtAll,
            spans: smallvec![],
            extra: ExtraErrInfo::at_context(ErrorContext::ParsingCommandLine).set_show_span(false).to_owned(),
        }
    }

    pub fn no_arg(kind: TokenKind, span: SpanRange) -> Self {
        ClapError {
            kind: ClapErrorKind::NoArg(kind),
            spans: smallvec![span],
            extra: ExtraErrInfo::at_context(ErrorContext::ParsingCommandLine),
        }
    }

    pub fn no_input_file() -> Self {
        ClapError {
            kind: ClapErrorKind::NoInputFile,
            spans: smallvec![],
            extra: ExtraErrInfo::at_context(ErrorContext::ParsingCommandLine).set_show_span(false).to_owned(),
        }
    }

    pub fn multiple_input_files(span1: SpanRange, span2: SpanRange) -> Self {
        ClapError {
            kind: ClapErrorKind::MultipleInputFiles,
            spans: smallvec![span1, span2],
            extra: ExtraErrInfo::at_context(ErrorContext::ParsingCommandLine),
        }
    }

    pub fn same_flag_multiple_times(flag: Option<Flag>, spans: SmallVec<[SpanRange; 1]>) -> Self {
        ClapError {
            kind: ClapErrorKind::SameFlagMultipleTimes(flag),
            spans,
            extra: ExtraErrInfo::at_context(ErrorContext::ParsingCommandLine),
        }
    }

    pub fn incompatible_flags(
        flag1: Flag,
        span1: SpanRange,
        flag2: Flag,
        span2: SpanRange,
    ) -> Self {
        ClapError {
            kind: ClapErrorKind::IncompatibleFlags(flag1, flag2),
            spans: smallvec![span1, span2],
            extra: ExtraErrInfo::at_context(ErrorContext::ParsingCommandLine),
        }
    }

    // `start` is inclusive, and `end` is exclusive
    pub fn integer_range_error(start: Option<BigInt>, end: Option<BigInt>, given: BigInt, span: SpanRange) -> Self {
        ClapError {
            kind: ClapErrorKind::IntegerRangeError {
                start, end, given,
            },
            spans: smallvec![span],
            extra: ExtraErrInfo::at_context(ErrorContext::ParsingCommandLine),
        }
    }
}

impl SodigyError<ClapErrorKind> for ClapError {
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

    fn error_kind(&self) -> &ClapErrorKind {
        &self.kind
    }

    fn index(&self) -> u32 {
        2
    }

    fn get_stage(&self) -> Stage {
        Stage::Clap
    }
}

pub enum ClapErrorKind {
    InvalidUtf8,
    InvalidArgument(TokenKind, String),
    NoArgsAtAll,
    NoArg(TokenKind),
    NoInputFile,
    MultipleInputFiles,
    IncompatibleFlags(Flag, Flag),

    // `None` indicates a cli option that does not have a flag: input_path
    SameFlagMultipleTimes(Option<Flag>),
    IntegerRangeError {
        start: Option<BigInt>,  // inclusive
        end: Option<BigInt>,    // exclusive
        given: BigInt,
    },
}

impl SodigyErrorKind for ClapErrorKind {
    fn msg(&self, _: &mut InternSession) -> String {
        match self {
            ClapErrorKind::InvalidUtf8 => String::from("invalid utf-8"),
            ClapErrorKind::NoArgsAtAll => String::from("expected an input file, got nothing"),
            ClapErrorKind::InvalidArgument(kind, arg) => format!(
                "expected {}, got `{}`",
                kind.render_error(),
                trim_long_string(arg.to_string(), 16, 16),
            ),
            ClapErrorKind::NoArg(kind) => format!(
                "expected {}, got nothing",
                kind.render_error(),
            ),
            ClapErrorKind::NoInputFile => String::from("no input file"),
            ClapErrorKind::MultipleInputFiles => String::from("multiple input files"),
            ClapErrorKind::SameFlagMultipleTimes(flag) => match flag {
                Some(flag) => format!("`{}` given more than once", flag.render_error()),
                None => "<INPUT> given more than once".to_string(),
            },
            ClapErrorKind::IncompatibleFlags(flag1, flag2) => format!("`{}` and `{}` are incompatible", flag1.render_error(), flag2.render_error()),
            ClapErrorKind::IntegerRangeError { start, end, given } => format!(
                "expected an integer in range {}..{}, got {given}",
                start.as_ref().map(|n| n.to_string()).unwrap_or(String::new()),
                end.as_ref().map(|n| n.to_string()).unwrap_or(String::new()),
            ),
        }
    }

    fn help(&self, _: &mut InternSession) -> String {
        match self {
            ClapErrorKind::NoArgsAtAll => String::from("Try `sodigy --help` to see available options."),
            ClapErrorKind::IncompatibleFlags(flag1, flag2) => match (flag1, flag2) {
                (Flag::Hir, Flag::Mir)
                | (Flag::Mir, Flag::Hir) => format!(
                    "`{}` asks the compiler to stop at the hir pass and `{}` asks the compiler to stop at the mir pass. Where should it stop at?",
                    Flag::Hir.render_error(),
                    Flag::Mir.render_error(),
                ),
                (Flag::Help, f)
                | (f, Flag::Help) => format!(
                    "There's no help message for `{}`",
                    f.render_error(),
                ),
                (Flag::Version, f)
                | (f, Flag::Version) => format!(
                    "There's no version info for `{}`",
                    f.render_error(),
                ),
                _ => String::new(),
            },
            ClapErrorKind::InvalidArgument(kind, argument) if *kind == TokenKind::Path &&
                (argument.starts_with("-") || argument.starts_with("=")) => format!(
                    "It's not allowed to use paths that start with \"{}\", in order to prevent confusion. If you want to do so, try \"./{argument}\"",
                    argument.chars().next().unwrap(),
                ),
            ClapErrorKind::SameFlagMultipleTimes(None) => String::from("Sodigy compiler cannot compile multiple files at once. You have to use `-L` option."),
            ClapErrorKind::InvalidArgument(_, _)
            | ClapErrorKind::InvalidUtf8
            | ClapErrorKind::NoArg(_)
            | ClapErrorKind::NoInputFile
            | ClapErrorKind::MultipleInputFiles
            | ClapErrorKind::SameFlagMultipleTimes(_)
            | ClapErrorKind::IntegerRangeError { .. } => String::new(),
        }
    }

    // we don't need this, but I want it to look more complete
    fn index(&self) -> u32 {
        match self {
            ClapErrorKind::InvalidUtf8 => 0,
            ClapErrorKind::InvalidArgument(_, _) => 1,
            ClapErrorKind::NoArgsAtAll => 2,
            ClapErrorKind::NoArg(_) => 3,
            ClapErrorKind::NoInputFile => 4,
            ClapErrorKind::MultipleInputFiles => 5,
            ClapErrorKind::SameFlagMultipleTimes(_) => 6,
            ClapErrorKind::IncompatibleFlags(_, _) => 7,
            ClapErrorKind::IntegerRangeError { .. } => 8,
        }
    }
}
