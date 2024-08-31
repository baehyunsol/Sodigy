use super::{
    DestructuredPattern,
    NumberLike,
    Pattern,
    PatternKind,
    RangeType,
    check_range_pattern,
    string::{StringPattern, lower_string_pattern},
};
use crate::lower_ast_ty;
use crate::error::HirError;
use crate::names::{IdentWithOrigin, NameSpace};
use crate::session::HirSession;
use crate::warn::HirWarning;
use sodigy_ast::{self as ast, FieldKind, IdentWithSpan};
use sodigy_intern::InternedString;
use sodigy_session::SodigySession;
use sodigy_span::SpanRange;
use std::collections::{HashMap, HashSet};

// `let pattern Foo { bar: $x, baz: $y } = f();`
// -> `let tmp = f();`, `let x = tmp.bar;`, `let y = tmp.baz;`
//
// `let pattern ($x, $y, .., $z, _) = f();`
// -> `let tmp = f();`, `let x = tmp._0;`, `let y = tmp._1;`, `let z = index(tmp, -1)`
//
// `let pattern ($x, ($y, $z), .., $w) = f();`
// -> `let tmp = f();`, `let x = tmp._0;`, `let tmp2 = tmp._1;`, `let y = tmp2._0;`, `let z = tmp2._1;`, `let w = index(tmp, -1)`
//
// `let pattern Foo { $x, $z @ .. } = f();`
// -> Invalid: No bindings for shorthand in this case
//
// `let pattern Foo { bar: $x @ Foo { .. }, .. } = f();`
// -> same as `let Foo { bar: $x, .. } = f();`
//
// let's not allow `$x @ _` -> it makes sense but why would someone do this?

// TODO: it doesn't check whether there are name collisions or not
pub fn lower_patterns_to_name_bindings(
    pattern: &ast::Pattern,
    expr: &ast::Expr,
    name_bindings: &mut Vec<DestructuredPattern>,
    session: &mut HirSession,
) -> Result<(), ()> {
    match &pattern.kind {
        ast::PatternKind::Binding(name) => {
            name_bindings.push(DestructuredPattern::new(
                IdentWithSpan::new(*name, pattern.span),
                expr.clone(),
                pattern.ty.clone(),
                true,
            ));
        },
        // let pattern ($x, ($y, $z)) = foo();
        // -> `let tmp = foo(); let x = tmp._0; let tmp2 = tmp._1; let y = tmp2._0; let z = tmp2._1;`
        ast::PatternKind::Tuple(patterns) => {
            let mut has_error = false;
            let tmp_name = session.allocate_tmp_name();

            // let tmp = foo();
            name_bindings.push(DestructuredPattern::new(
                IdentWithSpan::new(tmp_name, pattern.span.into_fake()),  // $tmp
                expr.clone(),
                pattern.ty.clone(),
                false,  // not a real name
            ));

            let name_bindings_len = name_bindings.len();
            let mut shorthand_index = None;

            for (index, curr_pattern) in patterns.iter().enumerate() {
                if curr_pattern.is_wildcard() {
                    if let Some(bind) = &curr_pattern.bind {
                        session.push_warning(
                            HirWarning::name_binding_on_wildcard(*bind)
                        );
                    }

                    continue;
                }

                if curr_pattern.is_shorthand() {
                    if let Some(_) = shorthand_index {
                        session.push_error(HirError::multiple_shorthands(
                            // It's okay to be inefficient when there's an error
                            get_all_shorthand_spans(&patterns)
                        ));
                        has_error = true;
                    }

                    else {
                        shorthand_index = Some(index);

                        // `let pattern (_, $x @ .., _) = (0, 1, 2, 3);`
                        // -> `let x = (1, 2);` or `let x = tmp.range(1, -1)`
                        if let Some(bind) = &curr_pattern.bind {
                            name_bindings.push(DestructuredPattern::new(
                                *bind,
                                field_expr_with_name_and_index(
                                    tmp_name,
                                    FieldKind::Range(index as i64, index as i64 - patterns.len() as i64 + 1),
                                    curr_pattern.span.into_fake(),
                                ),
                                None,
                                false,  // TODO: is this real?
                            ));
                        }
                    }

                    continue;
                }

                let subpattern_expr = if let Some(_) = shorthand_index {
                    // `(_, _, .., $x, $y)`
                    // `$x` -> `tuple_field_index(tmp, -2)`
                    field_expr_with_name_and_index(
                        tmp_name,
                        FieldKind::Index(index as i64 - patterns.len() as i64),
                        curr_pattern.span.into_fake(),
                    )
                } else {
                    // `tmp` + 0 -> `tmp._0`
                    field_expr_with_name_and_index(
                        tmp_name,
                        FieldKind::Index(index as i64),
                        curr_pattern.span.into_fake(),
                    )
                };

                if let Err(()) = lower_patterns_to_name_bindings(
                    curr_pattern,  // $x
                    &subpattern_expr,  // tmp._0
                    name_bindings,
                    session,
                ) {
                    has_error = true;
                }
            }

            if has_error {
                return Err(());
            }
        },
        // let pattern Foo { x: $x, y: Bar { y: $y, z: $z }, .. } = foo();
        // ->
        // let tmp = foo();
        // let x = tmp.x;
        // let pattern Bar { y: $y, z: $z } = tmp.y;
        // ->
        // let tmp = foo();
        // let x = tmp.x;
        // let tmp2 = tmp.y;
        // let y = tmp2.y;
        // let z = tmp2.z;
        ast::PatternKind::Struct {
            struct_name,
            fields,
            ..
        } => {
            let mut has_error = false;
            let tmp_name = session.allocate_tmp_name();

            // let tmp = foo();
            name_bindings.push(DestructuredPattern::new(
                IdentWithSpan::new(tmp_name, pattern.span.into_fake()),  // $tmp
                expr.clone(),
                pattern.ty.clone(),
                false,  // not a real name
            ));

            for ast::PatField {
                name,
                pattern,
            } in fields.iter() {
                if pattern.is_wildcard() || pattern.is_shorthand() {
                    continue;
                }

                let subpattern_expr = field_expr_with_name_and_index(
                    tmp_name,
                    FieldKind::Named(*name),
                    pattern.span.into_fake(),
                );

                if let Err(()) = lower_patterns_to_name_bindings(
                    pattern,      // x
                    &subpattern_expr,  // tmp.x
                    name_bindings,
                    session,
                ) {
                    has_error = true;
                }
            }

            if has_error {
                return Err(());
            }
        },
        _ => {
            session.push_error(HirError::refutable_pattern_in_let(pattern));
            return Err(());
        },
    }

    Ok(())
}

// TODO: `(p1, p2, p3 | p4)` -> `(p1, p2, p3) | (p1, p2, p4)`
// TODO: does it make sense? that would explode exponentially
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
            from, to,
            inclusive,
            is_string,
        } => {
            // "abc".."def" -> prefix and suffix patterns
            if *is_string {
                // inclusive ranges are not allowed for string patterns
                if *inclusive {
                    session.push_error(HirError::inclusive_string_pattern(span));
                    return Err(());
                }

                let mut result = StringPattern::new();

                lower_string_pattern(
                    from, to,
                    session,
                    &mut result,
                )?;

                PatternKind::String(result)
            }

            // 'a'..~'z' or 0..9
            else {
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
            }
        },
        ast::PatternKind::Tuple(patterns) => {
            let mut result = Vec::with_capacity(patterns.len());
            let mut has_error = false;

            for pattern in patterns.iter() {
                if let Ok(pattern) = lower_ast_pattern(
                    pattern,
                    session,
                    used_names,
                    imports,
                    name_space,
                ) {
                    result.push(pattern);
                }

                else {
                    has_error = true;
                }
            }

            if has_error {
                return Err(());
            }

            PatternKind::Tuple(result)
        },
        ast::PatternKind::TupleStruct { name, fields } => {
            let mut result = Vec::with_capacity(fields.len());
            let mut has_error = false;

            for pattern in fields.iter() {
                if let Ok(pattern) = lower_ast_pattern(
                    pattern,
                    session,
                    used_names,
                    imports,
                    name_space,
                ) {
                    result.push(pattern);
                }

                else {
                    has_error = true;
                }
            }

            if has_error {
                return Err(());
            }

            PatternKind::TupleStruct { name: name.to_vec(), fields: result }
        },
        ast::PatternKind::Wildcard => PatternKind::Wildcard,
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
            ast::PatternKind::Number(n1),
            ast::PatternKind::Number(n2),
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

/// `'name'` + `0` -> `name._0`
fn field_expr_with_name_and_index(name: InternedString, index: FieldKind, span: SpanRange) -> ast::Expr {
    ast::Expr {
        kind: ast::ExprKind::Field {
            pre: Box::new(ast::Expr {
                kind: ast::ExprKind::Value(ast::ValueKind::Identifier(name)),
                span,
            }),
            post: index,
        },
        span,
    }
}

fn get_all_shorthand_spans(patterns: &[ast::Pattern]) -> Vec<SpanRange> {
    patterns.iter().filter(
        |pat| matches!(pat.kind, ast::PatternKind::Shorthand)
    ).map(
        |pat| pat.span
    ).collect()
}
