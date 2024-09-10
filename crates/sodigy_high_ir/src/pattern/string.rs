use crate::HirSession;
use crate::error::HirError;
use sodigy_ast as ast;
use sodigy_parse::IdentWithSpan;
use sodigy_session::SodigySession;

#[derive(Clone)]
pub struct StringPattern {
    pub(crate) strings: Vec<IdentWithSpan>,

    // .."a"
    pub(crate) open_prefix: bool,

    // "a"..
    pub(crate) open_suffix: bool,

    // prefix `b`
    pub(crate) is_binary: bool,
}

impl StringPattern {
    pub fn new() -> Self {
        StringPattern {
            strings: vec![],
            open_prefix: false,
            open_suffix: false,
            is_binary: false,
        }
    }
}

// extracts `["abc", "def", "ghi", None]` from `"abc".."def".."ghi"..`
// it also checks types
// it rejects all the name bindings and type annotations
pub fn lower_string_pattern(
    pre: &Option<Box<ast::Pattern>>,
    post: &Option<Box<ast::Pattern>>,
    session: &mut HirSession,
    result: &mut StringPattern,
) -> Result<(), ()> {
    match pre {
        None => {
            result.open_prefix = true;
        },
        Some(prefix) => {
            let prefix = prefix.as_ref();
            reject_name_binding_and_type_anno(prefix, session)?;

            match prefix {
                ast::Pattern {
                    kind: ast::PatternKind::String {
                        content, is_binary
                    },
                    span,
                    ..
                } => {
                    result.is_binary = *is_binary;
                    result.strings.push(IdentWithSpan::new(*content, *span));
                },
                ast::Pattern {
                    kind: ast::PatternKind::Range {
                        from, to, inclusive, ..
                    },
                    span,
                    ..
                } => {
                    if *inclusive {
                        session.push_error(HirError::inclusive_string_pattern(*span));
                        return Err(());
                    }

                    lower_string_pattern(from, to, session, result)?;
                },
                ast::Pattern {
                    span, ..
                } => {
                    session.push_error(HirError::ty_error(vec![*span]));
                    return Err(());
                }
            }
        },
    }

    match post {
        None => {
            result.open_suffix = true;
        },
        Some(suffix) => {
            let suffix = suffix.as_ref();
            reject_name_binding_and_type_anno(suffix, session)?;

            match suffix {
                ast::Pattern {
                    kind: ast::PatternKind::String {
                        content, is_binary
                    },
                    span,
                    ..
                } => {
                    if result.is_binary != *is_binary {
                        session.push_error(HirError::ty_error(vec![*result.strings[0].span(), *span]));
                        return Err(());
                    }

                    result.strings.push(IdentWithSpan::new(*content, *span));
                },
                ast::Pattern {
                    kind: ast::PatternKind::Range {
                        from, to, inclusive, ..
                    },
                    span,
                    ..
                } => {
                    if *inclusive {
                        session.push_error(HirError::inclusive_string_pattern(*span));
                        return Err(());
                    }

                    lower_string_pattern(from, to, session, result)?;
                },
                ast::Pattern {
                    span, ..
                } => {
                    session.push_error(HirError::ty_error(vec![*span]));
                    return Err(());
                }
            }
        },
    }

    Ok(())
}

fn reject_name_binding_and_type_anno(
    pattern: &ast::Pattern,
    session: &mut HirSession,
) -> Result<(), ()> {
    if let Some(ty) = &pattern.ty {
        session.push_error(HirError::ty_anno_not_allowed_here(ty.as_expr().span));
        return Err(());
    }

    if let Some(name) = &pattern.bind {
        session.push_error(HirError::name_binding_not_allowed_here(*name.span()));
        return Err(());
    }

    Ok(())
}
