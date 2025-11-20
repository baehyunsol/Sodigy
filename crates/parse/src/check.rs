use crate::{CallArg, Session};
use sodigy_error::{Error, ErrorKind};
use sodigy_span::RenderableSpan;

// If new names are defined (e.g. function params, struct field defs), it checks name collisions.
// If defined names are used (e.g. calling a function with keyword args, initializing a struct), it doesn't check name collisions.

mod attribute;
mod block;
mod r#enum;
mod expr;
mod func;
mod r#if;
mod r#let;
mod r#match;
mod module;
mod pattern;
mod r#struct;
mod r#type;
mod r#use;

pub(crate) fn check_call_args(args: &[CallArg], session: &Session) -> Result<(), Vec<Error>> {
    // Like Python, a positional argument cannot follow a keyword argument
    let mut has_to_be_kwarg = false;
    let mut keyword_span = None;
    let mut errors = vec![];

    for arg in args.iter() {
        // It doesn't check the name collisions in keyword args -> will be done later.
        if has_to_be_kwarg && arg.keyword.is_none() {
            errors.push(Error {
                kind: ErrorKind::PositionalArgAfterKeywordArg,
                spans: vec![
                    RenderableSpan {
                        span: arg.arg.error_span(),
                        auxiliary: false,
                        note: Some(String::from("A positional argument cannot come after a keyword argument.")),
                    },
                    RenderableSpan {
                        span: keyword_span.unwrap(),
                        auxiliary: true,
                        note: Some(String::from("We have a keyword argument here.")),
                    },
                ],
                ..Error::default()
            });
        }

        if let Err(e) = arg.arg.check(session) {
            errors.extend(e);
        }

        if let Some((_, span)) = arg.keyword {
            has_to_be_kwarg = true;
            keyword_span = Some(span);
        }
    }

    if errors.is_empty() {
        Ok(())
    }

    else {
        Err(errors)
    }
}
