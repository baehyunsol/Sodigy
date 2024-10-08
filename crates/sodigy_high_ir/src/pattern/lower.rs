use super::{
    DestructuredPattern,
    ExprKind,
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
use sodigy_ast::{self as ast, FieldKind};
use sodigy_intern::InternedString;
use sodigy_parse::IdentWithSpan;
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

pub fn lower_patterns_to_name_bindings(
    pattern: &ast::Pattern,
    expr: &ast::Expr,
    name_bindings: &mut Vec<DestructuredPattern>,
    session: &mut HirSession,
) -> Result<(), ()> {
    match &pattern.kind {
        // let pattern $x = foo();
        // -> `let x = foo();`  (no change)
        ast::PatternKind::Binding(name) => {
            name_bindings.push(DestructuredPattern::new(
                IdentWithSpan::new(*name, pattern.span),
                expr.clone(),
                pattern.ty.clone(),
                true,
            ));
        },
        // `let pattern $x @ _ = foo();` ||
        // `let pattern $x @ 1 = foo();` ||
        // `let pattern $x @ a = foo();` || ...
        // -> `let x = foo();`
        // this function doesn't care whether `foo()` matches the pattern or not
        // it would either be checked by refutability check or runtime pattern matching
        ast::PatternKind::Wildcard
        | ast::PatternKind::Identifier(_)
        | ast::PatternKind::Number(_)
        | ast::PatternKind::Char(_)
        | ast::PatternKind::String { .. } => {
            if let Some(name) = &pattern.bind {
                name_bindings.push(DestructuredPattern::new(
                    IdentWithSpan::new(name.id(), *name.span()),
                    expr.clone(),
                    pattern.ty.clone(),
                    true,
                ));
            }
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
                                false,
                            ));
                        }
                    }

                    continue;
                }

                let subpattern_expr = if let Some(_) = shorthand_index {
                    // `let pattern (_, _, .., $x, $y) = ...`
                    // `$x` -> `tuple_field_index(tmp, -2)`
                    field_expr_with_name_and_index(
                        tmp_name,
                        FieldKind::Index(index as i64 - patterns.len() as i64),
                        curr_pattern.span.into_fake(),
                    )
                } else {
                    // `let pattern ($x, _, ..) = ...`
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
        // similar to tuple
        // let pattern [$x, [$y, $z]] = foo();
        // -> `let tmp = foo(); let x = tmp.[0]; let tmp2 = tmp.[1]; let y = tmp2.[0]; let z = tmp2.[1];`
        ast::PatternKind::List(patterns) => {
            let mut has_error = false;
            let tmp_name = session.allocate_tmp_name();

            // let tmp = foo();
            name_bindings.push(DestructuredPattern::new(
                IdentWithSpan::new(tmp_name, pattern.span.into_fake()),  // $tmp
                expr.clone(),
                pattern.ty.clone(),
                false,  // not a real name
            ));

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

                        // `let pattern [_, $x @ .., _] = [0, 1, 2, 3];`
                        // -> `let x = tmp[1 .. -1]`
                        if let Some(bind) = &curr_pattern.bind {
                            name_bindings.push(DestructuredPattern::new(
                                *bind,
                                index_expr_with_name_and_index(
                                    tmp_name,
                                    index as i64,
                                    Some(index as i64 - patterns.len() as i64 + 1),
                                    curr_pattern.span.into_fake(),
                                    session,
                                ),
                                None,
                                false,
                            ));
                        }
                    }

                    continue;
                }

                let subpattern_expr = if let Some(_) = shorthand_index {
                    // `let pattern [_, _, .., $x, $y] = ...`
                    // -> `let x = tmp[-2];`
                    index_expr_with_name_and_index(
                        tmp_name,
                        index as i64 - patterns.len() as i64,
                        None,
                        curr_pattern.span.into_fake(),
                        session,
                    )
                } else {
                    // `let pattern [_, $x, ..] = ...`
                    // -> `let x = tmp[1];`
                    index_expr_with_name_and_index(
                        tmp_name,
                        index as i64,
                        None,
                        curr_pattern.span.into_fake(),
                        session,
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
        _ => {
            session.push_error(HirError::todo(&format!("lower_patterns_to_name_bindings({pattern:?})"), pattern.span));
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
        p_kind @ (ast::PatternKind::Tuple(patterns)
        | ast::PatternKind::List(patterns)
        | ast::PatternKind::Or(patterns)) => {
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

            if let ast::PatternKind::Tuple(_) = p_kind {
                PatternKind::Tuple(result)
            }

            else if let ast::PatternKind::List(_) = p_kind {
                PatternKind::List(result)
            }

            else {
                PatternKind::Or(result)
            }
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
        ast::PatternKind::Shorthand => PatternKind::Shorthand,

        // these two basically have the same meaning: an enum variant without values
        ast::PatternKind::Identifier(id) => PatternKind::TupleStruct { name: vec![IdentWithSpan::new(*id, span)], fields: vec![] },
        ast::PatternKind::Path(name) => PatternKind::TupleStruct { name: name.to_vec(), fields: vec![] },

        ast::PatternKind::Number(n) => if n.is_integer() {
            PatternKind::Constant(ExprKind::Integer(*n))
        } else {
            PatternKind::Constant(ExprKind::Ratio(*n))
        },
        ast::PatternKind::Char(c) => PatternKind::Constant(ExprKind::Char(*c)),
        ast::PatternKind::String { content, is_binary } => PatternKind::Constant(ExprKind::String {
            content: *content,
            is_binary: *is_binary,
        }),
        ast::PatternKind::Struct { .. } => {
            session.push_error(HirError::todo("struct patterns", span));
            return Err(());
        },
        ast::PatternKind::OrRaw(_, _) => unreachable!(),
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
            ast::PatternKind::Number(n1),
            ast::PatternKind::Number(n2),
        ) => {
            session.push_error(HirError::type_error(
                vec![p1.span, p2.span],
                p1.get_type_string(),  // expected
                p2.get_type_string(),  // got
            ));

            return Err(());
        },
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
            session.push_error(HirError::type_error(
                vec![p2.span],
                p1.get_type_string(),  // expected
                p2.get_type_string(),  // got
            ));

            Err(())
        },
        _ => {
            // invalid types for a range pattern
            session.push_error(HirError::type_error(
                vec![p1.span, p2.span],
                // TODO: better representation?
                String::from("Int | Ratio | String | Char"),  // expected

                // TODO: error message doesn't make sense when p1 is a valid type and p2 is not
                p1.get_type_string(),  // got
            ));

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

/// ('name', 1, None) -> `name[1]`
/// ('name', -3, Some(-1)) -> `name[-3..-1]`
fn index_expr_with_name_and_index(
    name: InternedString,
    index_start: i64,
    index_end: Option<i64>,
    span: SpanRange,
    session: &mut HirSession,
) -> ast::Expr {
    let index_rhs = if let Some(index_end) = index_end {
        ast::Expr {
            kind: ast::ExprKind::InfixOp(
                ast::InfixOp::Range,
                Box::new(ast::Expr {
                    kind: ast::ExprKind::Value(ast::ValueKind::Number(session.intern_numeric(index_start.into()))),
                    span,
                }),
                Box::new(ast::Expr {
                    kind: ast::ExprKind::Value(ast::ValueKind::Number(session.intern_numeric(index_end.into()))),
                    span,
                }),
            ),
            span,
        }
    } else {
        ast::Expr {
            kind: ast::ExprKind::Value(ast::ValueKind::Number(session.intern_numeric(index_start.into()))),
            span,
        }
    };

    ast::Expr {
        kind: ast::ExprKind::InfixOp(
            ast::InfixOp::Index,
            Box::new(ast::Expr {
                kind: ast::ExprKind::Value(ast::ValueKind::Identifier(name)),
                span,
            }),
            Box::new(index_rhs),
        ),
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

pub fn check_names_in_or_patterns(pattern: &ast::Pattern) -> Vec<HirError> {
    match &pattern.kind {
        ast::PatternKind::Or(patterns) => {
            // it has to keep spans for error messages
            let mut name_set: HashMap<InternedString, SpanRange> = HashMap::new();
            let first_pattern_span = patterns[0].span;
            let mut errors = vec![];

            for (index, pattern) in patterns.iter().enumerate() {
                let mut buffer = vec![];
                pattern.get_name_bindings(&mut buffer);

                if index == 0 {
                    name_set = buffer.into_iter().map(
                        |name| (name.id(), *name.span())
                    ).collect();
                }

                else {
                    let mut name_collision_checker = HashMap::new();

                    for name in buffer.iter() {
                        if name_collision_checker.contains_key(&name.id()) {
                            errors.push(HirError::name_collision(
                                IdentWithSpan::new(name.id(), *name.span()),
                                IdentWithSpan::new(name.id(), *name_collision_checker.get(&name.id()).unwrap()),
                            ));
                            continue;
                        }

                        else {
                            name_collision_checker.insert(name.id(), *name.span());
                        }

                        if !name_set.contains_key(&name.id()) {
                            errors.push(HirError::name_not_bound_in_all_patterns(
                                *name,
                                first_pattern_span,
                            ));
                        }
                    }

                    for (name, span) in name_set.iter() {
                        if !name_collision_checker.contains_key(name) {
                            errors.push(HirError::name_not_bound_in_all_patterns(
                                IdentWithSpan::new(*name, *span),
                                pattern.span,
                            ));
                        }
                    }
                }
            }

            errors
        },
        _ => vec![],  // no error
    }
}
