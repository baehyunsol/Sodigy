use super::{dump_assert, dump_let, dump_type, span_to_string_or_verbose};
use crate::{Callable, Expr, Session, Type};
use sodigy_endec::IndentedLines;
use sodigy_error::EnumFieldKind;
use sodigy_hir::dump::dump_pattern;
use sodigy_parse::Field;
use sodigy_session::SodigySession;
use sodigy_span::Span;
use std::collections::HashMap;

pub fn dump_expr<S: SodigySession>(
    expr: &Expr,
    lines: &mut IndentedLines,
    types: &HashMap<Span, Type>,
    session: &S,
    max_len: usize,
    single_line: bool,
) {
    if max_len != 0 && lines.total_chars() > max_len {
        return;
    }

    match expr {
        Expr::Ident { id, dotfish } => {
            lines.push(&id.id.unintern_or_default(session.intermediate_dir()));

            if let Some(dotfish) = dotfish {
                lines.push(".<");

                for r#type in dotfish.types.iter() {
                    dump_type(r#type, lines, session);
                }

                lines.push(">");
            }
        },
        Expr::Constant(c) => {
            lines.push(&c.dump(session.intermediate_dir()));
        },
        Expr::If(r#if) => {
            lines.push("if ");

            let if_cond = matches!(&*r#if.cond, Expr::If(_));

            if if_cond {
                lines.push("(");
            }

            dump_expr(&r#if.cond, lines, types, session, max_len, single_line);

            if if_cond {
                lines.push(")");
            }

            match get_if_value_dump_type(&r#if.true_value, types, session) {
                IfValueDumpType::LongExpr if !single_line => {
                    lines.push(" {");
                    lines.inc_indent();
                    lines.break_line();
                    dump_expr(&r#if.true_value, lines, types, session, max_len, single_line);
                    lines.dec_indent();
                    lines.break_line();
                    lines.push("} else");
                },
                IfValueDumpType::Block => {
                    lines.push(" ");
                    dump_expr(&r#if.true_value, lines, types, session, max_len, single_line);
                    lines.push(" else");
                },
                _ => {
                    lines.push(" { ");
                    dump_expr(&r#if.true_value, lines, types, session, max_len, single_line);
                    lines.push(" } else");
                },
            }

            match get_if_value_dump_type(&r#if.false_value, types, session) {
                IfValueDumpType::LongExpr if !single_line => {
                    lines.push(" {");
                    lines.inc_indent();
                    lines.break_line();
                    dump_expr(&r#if.false_value, lines, types, session, max_len, single_line);
                    lines.dec_indent();
                    lines.break_line();
                    lines.push("}");
                },
                IfValueDumpType::Block => {
                    lines.push(" ");
                    dump_expr(&r#if.false_value, lines, types, session, max_len, single_line);
                },
                _ => {
                    lines.push(" { ");
                    dump_expr(&r#if.false_value, lines, types, session, max_len, single_line);
                    lines.push(" }");
                },
            }
        },
        Expr::Match(r#match) => {
            if r#match.lowered_from_if {
                lines.push("/* lowered from if */");
            }

            lines.push("match ");
            dump_expr(&r#match.scrutinee, lines, types, session, max_len, single_line);
            lines.push(" {");

            if single_line {
                lines.push(" ");
            }

            else {
                lines.inc_indent();
                lines.break_line();
            }

            for arm in r#match.arms.iter() {
                dump_pattern(&arm.pattern, lines, session);

                if let Some(guard) = &arm.guard {
                    lines.push(" if ");
                    dump_expr(guard, lines, types, session, max_len, single_line);
                }

                lines.push(" => ");
                dump_expr(&arm.value, lines, types, session, max_len, single_line);
                lines.push(",");

                if single_line {
                    lines.break_line();
                }

                else {
                    lines.push(" ");
                }
            }

            if !single_line {
                lines.dec_indent();
                lines.break_line();
            }

            lines.push("}");
        },
        Expr::Block(block) => {
            lines.push("{");

            if single_line {
                lines.push(" ");
            }

            else {
                lines.inc_indent();
                lines.break_line();
            }

            for (i, r#let) in block.lets.iter().enumerate() {
                dump_let(r#let, lines, types, session, i != 0);
            }

            for assert in block.asserts.iter() {
                dump_assert(assert, lines, types, session);
            }

            dump_expr(&block.value, lines, types, session, max_len, single_line);

            if single_line {
                lines.push(" ");
            }

            else {
                lines.dec_indent();
                lines.break_line();
            }

            lines.push("}");
        },
        Expr::Field { lhs, fields, dotfish } => {
            dump_expr(lhs, lines, types, session, max_len, single_line);

            if let Some(dotfish) = &dotfish[0] {
                lines.push(".<");

                for r#type in dotfish.types.iter() {
                    dump_type(r#type, lines, session);
                }

                lines.push(">");
            }

            assert_eq!(fields.len() + 1, dotfish.len());

            for (field, dotfish) in fields.iter().zip(dotfish[1..].iter()) {
                lines.push(".");

                match field {
                    Field::Name { name, .. } => {
                        lines.push(&name.unintern_or_default(session.intermediate_dir()));
                    },
                    Field::Index(i) => {
                        if *i < 0 { todo!() }
                        lines.push(&format!("_{i}"));
                    },
                    Field::Range(_, _) => todo!(),
                    Field::EnumDiscriminant => {
                        lines.push("__DISCRIMINANT__");
                    },
                    Field::ListLength => {
                        lines.push("__LIST_LENGTH__");
                    },
                }

                if let Some(dotfish) = dotfish {
                    lines.push(".<");

                    for r#type in dotfish.types.iter() {
                        dump_type(r#type, lines, session);
                    }

                    lines.push(">");
                }
            }
        },
        Expr::FieldUpdate { lhs, fields, rhs } => {
            dump_expr(lhs, lines, types, session, max_len, single_line);
            lines.push(" `");

            for (i, field) in fields.iter().enumerate() {
                match field {
                    Field::Name { name, .. } => {
                        if i != 0 {
                            lines.push(".");
                        }

                        lines.push(&name.unintern_or_default(session.intermediate_dir()));
                    },
                    Field::Index(n) => {
                        lines.push(&format!("[{n}]"));
                    },
                    _ => todo!(),
                }
            }

            lines.push(" ");
            dump_expr(rhs, lines, types, session, max_len, single_line);
        },
        Expr::Call { func, args, .. } => {
            let mut is_tuple = false;
            let (open_delim, close_delim) = match func {
                Callable::Static { def_span, .. } => {
                    lines.push(&span_to_string_or_verbose(
                        def_span,
                        session.intermediate_dir(),
                        session.span_string_map().unwrap_or(&HashMap::new()),
                    ));
                    ("(", ")")
                },
                // TODO: dump field names
                Callable::StructInit { def_span, .. } => {
                    lines.push(&span_to_string_or_verbose(
                        def_span,
                        session.intermediate_dir(),
                        session.span_string_map().unwrap_or(&HashMap::new()),
                    ));
                    ("{", "}")
                },
                Callable::EnumInit { parent_def_span, variant_def_span, kind, .. } => {
                    lines.push(&format!(
                        "{}.{}",
                        span_to_string_or_verbose(
                            parent_def_span,
                            session.intermediate_dir(),
                            session.span_string_map().unwrap_or(&HashMap::new()),
                        ),
                        span_to_string_or_verbose(
                            variant_def_span,
                            session.intermediate_dir(),
                            session.span_string_map().unwrap_or(&HashMap::new()),
                        ),
                    ));

                    // TODO: dump field names for `EnumFieldKind::Struct`
                    match kind {
                        EnumFieldKind::None => ("", ""),
                        EnumFieldKind::Tuple => ("(", ")"),
                        EnumFieldKind::Struct => ("{", "}"),
                    }
                },
                Callable::TupleInit { .. } => {
                    is_tuple = true;
                    ("(", ")")
                },
                Callable::ListInit { .. } => ("[", "]"),
                Callable::Dynamic(f) => {
                    lines.push("(");
                    dump_expr(f, lines, types, session, max_len, single_line);
                    lines.push(")");
                    ("(", ")")
                },
            };

            lines.push(open_delim);
            let arg_per_line = !single_line && lookahead_args(args, types, session, 21) > 20;
            let has_trailing_comma = is_tuple && args.len() == 1 || arg_per_line;

            if args.len() > 1 {
                if arg_per_line {
                    lines.inc_indent();
                    lines.break_line();
                }

                for (i, arg) in args.iter().enumerate() {
                    dump_expr(arg, lines, types, session, max_len, single_line);

                    if has_trailing_comma || i < args.len() - 1 {
                        lines.push(",");
                    }

                    if i != args.len() - 1 {
                        if arg_per_line {
                            lines.break_line();
                        }

                        else {
                            lines.push(" ");
                        }
                    }
                }

                if arg_per_line {
                    lines.dec_indent();
                    lines.break_line();
                }
            }

            else {
                for arg in args.iter() {
                    dump_expr(arg, lines, types, session, max_len, single_line);
                }
            }

            lines.push(close_delim);
        },
    }
}

fn lookahead_args<S: SodigySession>(args: &[Expr], types: &HashMap<Span, Type>, session: &S, max_len: usize) -> usize {
    let mut count = 0;

    for arg in args.iter() {
        let mut indented_lines = IndentedLines::new();
        dump_expr(arg, &mut indented_lines, types, session, max_len, false);
        count += indented_lines.dump().len();
    }

    count
}

enum IfValueDumpType {
    Block,
    ShortExpr,
    LongExpr,
}

fn get_if_value_dump_type<S: SodigySession>(value: &Expr, types: &HashMap<Span, Type>, session: &S) -> IfValueDumpType {
    match value {
        Expr::Block(_) => IfValueDumpType::Block,
        _ => {
            let mut indented_lines = IndentedLines::new();
            dump_expr(value, &mut indented_lines, types, session, 21, false);

            if indented_lines.dump().len() > 20 {
                IfValueDumpType::LongExpr
            } else {
                IfValueDumpType::ShortExpr
            }
        },
    }
}
