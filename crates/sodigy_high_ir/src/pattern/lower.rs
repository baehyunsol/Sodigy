use super::{NumberLike, Pattern, PatternKind, RangeType, check_range_pattern};
use crate::lower_ast_ty;
use crate::err::HirError;
use crate::names::{IdentWithOrigin, NameSpace};
use crate::session::HirSession;
use sodigy_ast::{self as ast, IdentWithSpan};
use sodigy_intern::InternedString;
use sodigy_span::SpanRange;
use std::collections::{HashMap, HashSet};

// `let Foo { bar: $x, baz: $y } = f();`
// -> `let $tmp = f();`, `let $x = $tmp.bar;`, `let $y = $tmp.baz;`
//
// `let Foo($x, $y, $z @ ..) = f();`
// -> TODO: `$z` should be a tuple!, there must be some kind of slice of tuples
//
// `let ($x, $y, .., $z, _) = f();`
// -> `let $tmp = f();`, `let $x = $tmp._0;`, `let $y = $tmp._1;`, `let $z = TODO`
// -> TODO: there must be some kind of slice of tuples for `$z`
//
// `let Foo { $x, $z @ .. } = f();`
// -> Invalid: No bindings for shorthand in this case
//
// `let Foo(Foo($x, $y), $z) = f();`
// -> `let $tmp = f();`, `let $tmp2 = $tmp._0;`, `let $x = $tmp2._0;`, `let $y = $tmp2._1;`, `let $z = $tmp._1;`
//
// `let Foo { bar: $x @ Foo { .. }, .. } = f();`
// -> same as `let Foo { bar: $x, .. } = f();`
//
// let's not allow `$x @ _` -> it makes sense but why would someone do this?
pub fn lower_patterns_to_name_bindings(
    pattern: &ast::Pattern,
    expr: &ast::Expr,

    // expressions are not lowered in this stage
    // the last element, boolean, indicates whether this name binding is
    // declared by user, or generated by the compiler
    name_bindings: &mut Vec<(IdentWithSpan, ast::Expr, bool)>,
    session: &mut HirSession,
) -> Result<(), ()> {
    match &pattern.kind {
        ast::PatternKind::Binding(name) => {
            // It's O(n), but `n` must be small enough in most cases
            for (prev, _, _) in name_bindings.iter() {
                if prev.id() == name {
                    session.push_error(HirError::name_collision(
                        *prev,
                        IdentWithSpan::new(*name, pattern.span),
                    ));

                    return Err(());
                }
            }

            name_bindings.push((IdentWithSpan::new(*name, pattern.span), expr.clone(), true));
        },
        ast::PatternKind::Tuple(patterns) => {
            for pattern in patterns.iter() {
                todo!();
            }
        },
        ast::PatternKind::TupleStruct {
            name,
            fields,
        } => {
            for pattern in fields.iter() {
                todo!();
            }
        },
        ast::PatternKind::Struct {
            struct_name,
            fields,
            ..
        } => {
            for ast::PatField {
                name,
                pattern,
            } in fields.iter() {
                todo!();
            }
        },
        _ => {
            session.push_error(HirError::refutable_pattern_in_let(pattern));
            return Err(());
        },
    }

    Ok(())
}

pub fn lower_ast_pattern(
    pattern: &ast::Pattern,
    session: &mut HirSession,
    used_names: &mut HashSet<IdentWithOrigin>,
    imports: &HashMap<InternedString, (SpanRange, Vec<IdentWithSpan>)>,
    name_space: &mut NameSpace,
) -> Result<Pattern, ()> {
    // make sure that both lower functions are called
    // regardless of compile errors

    let kind = lower_ast_pattern_kind(
        &pattern.kind,
        pattern.span,
        session,
        used_names,
        imports,
        name_space,
    );
    let ty = if let Some(ty) = &pattern.ty {
        Some(lower_ast_ty(
            &ty,
            session,
            used_names,
            imports,
            name_space,
        )?)
    } else {
        None
    };

    Ok(Pattern {
        span: pattern.span,
        bind: pattern.bind,
        ty,
        kind: kind?,
    })
}

fn lower_ast_pattern_kind(
    pattern_kind: &ast::PatternKind,
    span: SpanRange,  // for error messages
    session: &mut HirSession,
    used_names: &mut HashSet<IdentWithOrigin>,
    imports: &HashMap<InternedString, (SpanRange, Vec<IdentWithSpan>)>,
    name_space: &mut NameSpace,
) -> Result<PatternKind, ()> {
    let res = match pattern_kind {
        ast::PatternKind::Binding(name) => PatternKind::Binding(*name),
        ast::PatternKind::Range {
            from, to, inclusive,
        } => {
            let res = match (from.as_ref().map(|f| f.as_ref()), to.as_ref().map(|t| t.as_ref())) {
                (Some(f), None) => {
                    // `0..~` doesn't make sense, how can an open end be inclusive?
                    if *inclusive {
                        session.push_error(HirError::open_inclusive_range(span));
                        return Err(());
                    }

                    PatternKind::Range {
                        ty: RangeType::try_from_pattern(
                            f, session,
                        )?,
                        from: NumberLike::try_from_pattern(
                            f, session,
                            true,  /* is_inclusive */
                        )?,
                        to: NumberLike::OpenEnd {
                            is_negative: true,
                        },
                    }
                },
                (None, Some(t)) => {
                    // `..'a'` -> `'\0'..'a'`
                    // `..0` -> `-inf..0`
                    let from = if let ast::PatternKind::Char(_) = &t.kind {
                        NumberLike::zero()
                    } else {
                        NumberLike::OpenEnd {
                            is_negative: true,
                        }
                    };

                    PatternKind::Range {
                        ty: RangeType::try_from_pattern(
                            t, session,
                        )?,
                        from,
                        to: NumberLike::try_from_pattern(
                            t, session,
                            *inclusive,
                        )?,
                    }
                },
                (Some(f), Some(t)) => {
                    check_same_type_or_error(
                        f, t,
                        session,
                    )?;

                    PatternKind::Range {
                        ty: RangeType::try_from_pattern(
                            t, session,
                        )?,
                        from: NumberLike::try_from_pattern(
                            f, session,
                            true,  // `is_inclusive` only affects the other side of a range
                        )?,
                        to: NumberLike::try_from_pattern(
                            t, session,
                            *inclusive,
                        )?,
                    }
                },
                (None, None) => unreachable!(),
            };

            check_range_pattern(
                &res,
                span,
                session,
            )?;

            res
        },
        _ => {
            session.push_error(HirError::todo("patterns", span));
            return Err(());
        },
    };

    Ok(res)
}

// it's only for range patterns
// that means `p1` and `p2` must either be num or char
fn check_same_type_or_error(
    p1: &ast::Pattern,
    p2: &ast::Pattern,
    session: &mut HirSession,
) -> Result<(), ()> {
    match (&p1.kind, &p2.kind) {
        (
            ast::PatternKind::Number { num: n1, .. },
            ast::PatternKind::Number { num: n2, .. },
        ) if n1.is_integer() == n2.is_integer() => Ok(()),  // valid types
        (
            ast::PatternKind::Char(_),
            ast::PatternKind::Char(_),
        ) => Ok(()),  // valid types
        (
            ast::PatternKind::Number { .. },
            ast::PatternKind::Char(_),
        ) | (
            ast::PatternKind::Char(_),
            ast::PatternKind::Number { .. },
        ) => {
            // valid types for a range pattern,
            // but they have to be the same

            // TODO: raise a 'real' type error when they're implemented
            session.push_error(HirError::ty_error(vec![p2.span]));

            Err(())
        },
        _ => {
            // invalid types for a range pattern

            // TODO: raise a 'real' type error when they're implemented
            session.push_error(HirError::ty_error(vec![p1.span, p2.span]));

            Err(())
        },
    }
}
