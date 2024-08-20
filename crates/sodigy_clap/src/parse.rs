use crate::error::ClapError;
use crate::flag::Flag;
use sodigy_span::{SpanPoint, SpanRange};

pub struct FlagWithArg {
    flag: Option<Flag>,
    flag_span: Option<SpanRange>,
    arg: Option<Arg>,
    arg_span: Option<SpanRange>,
}

pub fn parse_cli(code: &[u8], span_start: SpanPoint) -> Result<Vec<FlagWithArg>, Vec<ClapError>> {
    let tokens = lex_cli(code, span_start);
    let mut curr_flag = None;
    let mut curr_flag_span = None;
    let mut curr_arg_kind = ArgKind::Optional(Box::new(ArgKind::Path));  // input file or another flag
    let mut result = vec![];

    for token in tokens.iter() {
        match curr_arg_kind {
            ArgKind::None => match Flag::try_parse(&token.buffer) {
                Some(flag) => {
                    result.push(FlagWithArg {
                        flag: curr_flag,
                        flag_span: curr_flag_span,
                        arg: None,
                        arg_span: None,
                    });
                    curr_flag = flag;
                },
                None => {
                    // err: invalid flag
                },
            },
            ArgKind::Optional(optional_arg_kind) => match Flag::try_parse(&token.buffer) {
                Some(flag) => {
                    result.push(FlagWithArg {
                        flag: curr_flag,
                        flag_span: curr_flag_span,
                        arg: None,
                        arg_span: None,
                    });
                    curr_flag = flag;
                },
                None => match optional_arg_kind.parse_single_token(&token) {
                    Ok(arg) => {
                        result.push(FlagWithArg {
                            flag: curr_flag,
                            flag_span: curr_flag_span,
                            arg,
                            arg_span: token.span,
                        });
                    },
                    Err(e) => {
                        // err
                    },
                },
            },
            ArgKind::Library => {
                // TODO: an argument with multiple tokens
            },
            _ => match curr_arg_kind.parse_single_token(&token) {
                Ok(arg) => {},
                Err(e) => {},
            },
        }
    }

    match curr_arg_kind {
        ArgKind::None
        | ArgKind::Optional(_) => {},
        _ => {},  // unexpected eof
    }

    Ok(result)
}
