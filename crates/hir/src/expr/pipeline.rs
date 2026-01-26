use sodigy_parse as ast;
use sodigy_span::Span;
use sodigy_string::InternedString;

pub fn replace_dollar(
    value: &mut ast::Expr,
    ident: InternedString,
    replaced_spans: &mut Vec<Span>,
    has_nested_pipeline: &mut bool,
) {
    match value {
        ast::Expr::Path(_) |
        ast::Expr::Number { .. } |
        ast::Expr::String { .. } |
        ast::Expr::Char { .. } |
        ast::Expr::Byte { .. } => {},
        ast::Expr::If(r#if) => {
            replace_dollar(
                &mut r#if.cond,
                ident,
                replaced_spans,
                has_nested_pipeline,
            );

            if let Some(pattern) = &mut r#if.pattern {
                replace_dollar_in_pattern(
                    pattern,
                    ident,
                    replaced_spans,
                    has_nested_pipeline,
                );
            }

            replace_dollar(
                &mut r#if.true_value,
                ident,
                replaced_spans,
                has_nested_pipeline,
            );
            replace_dollar(
                &mut r#if.false_value,
                ident,
                replaced_spans,
                has_nested_pipeline,
            );
        },
        ast::Expr::Match(r#match) => {
            replace_dollar(
                &mut r#match.scrutinee,
                ident,
                replaced_spans,
                has_nested_pipeline,
            );

            for arm in r#match.arms.iter_mut() {
                replace_dollar_in_pattern(
                    &mut arm.pattern,
                    ident,
                    replaced_spans,
                    has_nested_pipeline,
                );

                if let Some(guard) = &mut arm.guard {
                    replace_dollar(
                        guard,
                        ident,
                        replaced_spans,
                        has_nested_pipeline,
                    );
                }

                replace_dollar(
                    &mut arm.value,
                    ident,
                    replaced_spans,
                    has_nested_pipeline,
                );
            }
        },
        ast::Expr::Block(block) => todo!(),
        ast::Expr::Call { func, args, .. } => {
            replace_dollar(
                func,
                ident,
                replaced_spans,
                has_nested_pipeline,
            );

            for arg in args.iter_mut() {
                replace_dollar(
                    &mut arg.arg,
                    ident,
                    replaced_spans,
                    has_nested_pipeline,
                );
            }
        },
        ast::Expr::FormattedString { elements, .. } => {
            for element in elements.iter_mut() {
                if let ast::ExprOrString::Expr(expr) = element {
                    replace_dollar(
                        expr,
                        ident,
                        replaced_spans,
                        has_nested_pipeline,
                    );
                }
            }
        },
        ast::Expr::Tuple { elements, .. } | ast::Expr::List { elements, .. } => {
            for element in elements.iter_mut() {
                replace_dollar(
                    element,
                    ident,
                    replaced_spans,
                    has_nested_pipeline,
                );
            }
        },
        ast::Expr::StructInit { r#struct, fields, .. } => {
            replace_dollar(
                r#struct,
                ident,
                replaced_spans,
                has_nested_pipeline,
            );

            for field in fields.iter_mut() {
                replace_dollar(
                    &mut field.value,
                    ident,
                    replaced_spans,
                    has_nested_pipeline,
                );
            }
        },
        ast::Expr::Field { .. } => todo!(),
        ast::Expr::FieldUpdate { .. } => todo!(),
        ast::Expr::Lambda { .. } => todo!(),
        ast::Expr::PrefixOp { rhs: expr, .. } | ast::Expr::PostfixOp { lhs: expr, .. } => {
            replace_dollar(
                expr,
                ident,
                replaced_spans,
                has_nested_pipeline,
            );
        },
        ast::Expr::InfixOp { rhs, lhs, .. } => {
            replace_dollar(
                rhs,
                ident,
                replaced_spans,
                has_nested_pipeline,
            );
            replace_dollar(
                lhs,
                ident,
                replaced_spans,
                has_nested_pipeline,
            );
        },
        // `x |> (f($) |> g($))` -> the first `$` is `x`, the second one is `f(x)`.
        ast::Expr::Pipeline { values, .. } => {
            replace_dollar(
                values.get_mut(0).unwrap(),
                ident,
                replaced_spans,
                has_nested_pipeline,
            );
            *has_nested_pipeline = true;
        },
        ast::Expr::PipelineData(span) => {
            let id_span = *span;
            *value = ast::Expr::Path(ast::Path { id: ident, id_span, fields: vec![] });
            replaced_spans.push(id_span);
        },
    }
}

fn replace_dollar_in_pattern(
    pattern: &mut ast::Pattern,
    ident: InternedString,
    replaced_spans: &mut Vec<Span>,
    has_nested_pipeline: &mut bool,
) {
    match &mut pattern.kind {
        ast::PatternKind::Path(_) |
        ast::PatternKind::NameBinding { .. } |
        ast::PatternKind::Number { .. } |
        ast::PatternKind::String { .. } |
        ast::PatternKind::Regex { .. } |
        ast::PatternKind::Char { .. } |
        ast::PatternKind::Byte { .. } |
        ast::PatternKind::Wildcard(_) => {},
        ast::PatternKind::Struct { .. } => todo!(),
        ast::PatternKind::TupleStruct { elements, .. } |
        ast::PatternKind::Tuple { elements, .. } |
        ast::PatternKind::List { elements, .. } => {
            for element in elements.iter_mut() {
                replace_dollar_in_pattern(
                    element,
                    ident,
                    replaced_spans,
                    has_nested_pipeline,
                );
            }
        },
        ast::PatternKind::Range { lhs, rhs, .. } => {
            for pat in [lhs, rhs] {
                if let Some(pat) = pat {
                    replace_dollar_in_pattern(
                        pat,
                        ident,
                        replaced_spans,
                        has_nested_pipeline,
                    );
                }
            }
        },
        ast::PatternKind::InfixOp { lhs, rhs, .. } |
        ast::PatternKind::Or { lhs, rhs, .. } => {
            replace_dollar_in_pattern(
                lhs,
                ident,
                replaced_spans,
                has_nested_pipeline,
            );
            replace_dollar_in_pattern(
                rhs,
                ident,
                replaced_spans,
                has_nested_pipeline,
            );
        },
        ast::PatternKind::PipelineData(span) => {
            let id_span = *span;
            pattern.kind = ast::PatternKind::Path(ast::Path { id: ident, id_span, fields: vec![] });
            replaced_spans.push(id_span);
        },
    }
}
