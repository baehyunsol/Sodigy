use crate::CallArg;
use sodigy_error::{Error, ErrorKind};

// If new names are defined (e.g. function arg defs, struct field defs), it checks name collisions.
// If defined names are used (e.g. calling a function with keyword args, initializing a struct), it doesn't check name collisions.

mod attribute;
mod block;
mod r#enum;
mod expr;
mod func;
mod r#if;
mod r#let;
mod module;
mod pattern;
mod r#struct;
mod r#type;
mod r#use;

pub(crate) fn check_call_args(args: &[CallArg]) -> Result<(), Vec<Error>> {
    // Like Python, a positional argument cannot follow a keyword argument
    let mut has_to_be_kwarg = false;
    let mut errors = vec![];

    for arg in args.iter() {
        // It doesn't check the name collisions in keyword args -> will be done later.
        if has_to_be_kwarg && arg.keyword.is_none() {
            errors.push(Error {
                kind: ErrorKind::PositionalArgAfterKeywordArg,
                span: arg.arg.error_span(),
                ..Error::default()
            });
        }

        if let Err(e) = arg.arg.check() {
            errors.extend(e);
        }

        if arg.keyword.is_some() {
            has_to_be_kwarg = true;
        }
    }

    if errors.is_empty() {
        Ok(())
    }

    else {
        Err(errors)
    }
}
