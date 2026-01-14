use crate::{Attribute, Decorator, DecoratorArg};
use sodigy_error::{Error, ErrorKind};
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::InternedString;
use std::collections::hash_map::{Entry, HashMap};

impl Attribute {
    pub fn check(&self, intermediate_dir: &str) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        // TODO: how about doc comments?

        for decorator in self.decorators.iter() {
            if let Err(e) = decorator.check(intermediate_dir) {
                errors.extend(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        }

        else {
            Err(errors)
        }
    }
}

impl Decorator {
    pub fn check(&self, intermediate_dir: &str) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        if let Some(args) = &self.args {
            if let Err(e) = check_decorator_args(args, intermediate_dir) {
                errors.extend(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        }

        else {
            Err(errors)
        }
    }
}

// In this context, positional arguments can follow keyword arguments because they're just different.
fn check_decorator_args(args: &[DecoratorArg], intermediate_dir: &str) -> Result<(), Vec<Error>> {
    let mut errors = vec![];

    // name collision check
    let mut spans_by_name: HashMap<InternedString, Vec<Span>> = HashMap::new();

    for arg in args.iter() {
        if let Some((name, span)) = arg.keyword {
            match spans_by_name.entry(name) {
                Entry::Occupied(mut e) => {
                    e.get_mut().push(span);
                },
                Entry::Vacant(e) => {
                    e.insert(vec![span]);
                },
            }
        }

        if let Ok(expr) = &arg.expr {
            if let Err(e) = expr.check(intermediate_dir) {
                // It may or may not be an error, so we have to throw it lazily...
                todo!()
            }
        }

        if let Ok(r#type) = &arg.r#type {
            if let Err(e) = r#type.check() {
                // It may or may not be an error, so we have to throw it lazily...
                todo!()
            }
        }
    }

    for (name, spans) in spans_by_name.into_iter() {
        if spans.len() > 1 {
            errors.push(Error {
                kind: ErrorKind::KeywordArgumentRepeated(name),
                spans: spans.into_iter().map(
                    |span| RenderableSpan {
                        span,
                        auxiliary: false,
                        note: None,
                    }
                ).collect(),
                note: None,
            });
        }
    }

    if errors.is_empty() {
        Ok(())
    }

    else {
        Err(errors)
    }
}
