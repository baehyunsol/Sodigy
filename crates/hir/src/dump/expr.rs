use super::{dump_assert, dump_let, dump_pattern, dump_type};
use crate::{CallArg, Expr, Session};
use sodigy_endec::IndentedLines;
use sodigy_parse::Field;
use sodigy_token::InfixOp;

pub fn dump_expr(expr: &Expr, lines: &mut IndentedLines, session: &Session, max_len: usize) {
    if max_len != 0 && lines.total_chars() > max_len {
        return;
    }

    match expr {
        Expr::Path(path) | Expr::Closure { fp: path, .. } => {
            lines.push(&path.unintern_or_default(&session.intermediate_dir));
        },
        Expr::Constant(c) => {
            lines.push(&c.dump(&session.intermediate_dir));
        },
        Expr::If(r#if) => {
            lines.push("if ");

            if let Some(pattern) = &r#if.pattern {
                dump_pattern(pattern, lines, session);
                lines.push(" = ");
            }

            let if_cond = matches!(&*r#if.cond, Expr::If(_));

            if if_cond {
                lines.push("(");
            }

            dump_expr(&r#if.cond, lines, session, max_len);

            if if_cond {
                lines.push(")");
            }

            match get_if_value_dump_type(&r#if.true_value, session) {
                IfValueDumpType::Block => {
                    lines.push(" ");
                    dump_expr(&r#if.true_value, lines, session, max_len);
                    lines.push(" else");
                },
                IfValueDumpType::ShortExpr => {
                    lines.push(" { ");
                    dump_expr(&r#if.true_value, lines, session, max_len);
                    lines.push(" } else");
                },
                IfValueDumpType::LongExpr => {
                    lines.push(" {");
                    lines.inc_indent();
                    lines.break_line();
                    dump_expr(&r#if.true_value, lines, session, max_len);
                    lines.dec_indent();
                    lines.break_line();
                    lines.push("} else");
                },
            }

            match get_if_value_dump_type(&r#if.false_value, session) {
                IfValueDumpType::Block => {
                    lines.push(" ");
                    dump_expr(&r#if.false_value, lines, session, max_len);
                },
                IfValueDumpType::ShortExpr => {
                    lines.push(" { ");
                    dump_expr(&r#if.false_value, lines, session, max_len);
                    lines.push(" }");
                },
                IfValueDumpType::LongExpr => {
                    lines.push(" {");
                    lines.inc_indent();
                    lines.break_line();
                    dump_expr(&r#if.false_value, lines, session, max_len);
                    lines.dec_indent();
                    lines.break_line();
                    lines.push("}");
                },
            }
        },
        Expr::Match(r#match) => {
            lines.push("match ");
            dump_expr(&r#match.scrutinee, lines, session, max_len);
            lines.push(" {");
            lines.inc_indent();
            lines.break_line();

            for (i, arm) in r#match.arms.iter().enumerate() {
                dump_pattern(&arm.pattern, lines, session);

                if let Some(guard) = &arm.guard {
                    lines.push(" if ");
                    dump_expr(guard, lines, session, max_len);
                }

                lines.push(" => ");
                dump_expr(&arm.value, lines, session, max_len);
                lines.push(",");

                if i != r#match.arms.len() - 1 {
                    lines.break_line();
                }
            }

            lines.dec_indent();
            lines.break_line();
            lines.push("}");
        },
        Expr::Block(block) => {
            lines.push("{");
            lines.inc_indent();
            lines.break_line();

            for (i, r#let) in block.lets.iter().enumerate() {
                dump_let(r#let, lines, session, i != 0);
            }

            for assert in block.asserts.iter() {
                dump_assert(assert, lines, session);
            }

            dump_expr(&block.value, lines, session, max_len);
            lines.dec_indent();
            lines.break_line();
            lines.push("}");
        },
        Expr::Call { func, args, .. } => {
            match &**func {
                Expr::Path(p) => {
                    lines.push(&p.unintern_or_default(&session.intermediate_dir));
                },
                _ => {
                    lines.push("(");
                    dump_expr(func, lines, session, max_len);
                    lines.push(")");
                },
            }

            lines.push("(");
            let arg_per_line = lookahead_args(args, session, 21) > 20;

            if args.len() > 1 {
                if arg_per_line {
                    lines.inc_indent();
                    lines.break_line();
                }

                for (i, arg) in args.iter().enumerate() {
                    dump_expr(&arg.arg, lines, session, max_len);
                    lines.push(",");

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
                    dump_expr(&arg.arg, lines, session, max_len);
                }
            }

            lines.push(")");
        },
        Expr::FormattedString { .. } => {
            lines.push(&format!("/* TODO: dump formatted string {expr:?} */"));
        },
        Expr::Tuple { elements, .. } | Expr::List { elements, .. } => {
            let is_tuple = matches!(expr, Expr::Tuple { .. });
            lines.push(if is_tuple { "(" } else { "[" });
            let element_per_line = lookahead_exprs(elements, session, 21) > 20;

            if elements.len() > 1 {
                if element_per_line {
                    lines.inc_indent();
                    lines.break_line();
                }

                for (i, element) in elements.iter().enumerate() {
                    dump_expr(element, lines, session, max_len);
                    lines.push(",");

                    if i != elements.len() - 1 {
                        if element_per_line {
                            lines.break_line();
                        }

                        else {
                            lines.push(" ");
                        }
                    }
                }

                if element_per_line {
                    lines.dec_indent();
                    lines.break_line();
                }
            }

            else {
                for element in elements.iter() {
                    dump_expr(element, lines, session, max_len);
                }

                if is_tuple && elements.len() == 1 {
                    lines.push(",");
                }
            }

            lines.push(if is_tuple { ")" } else { "]" });
        },
        Expr::StructInit { .. } => {
            lines.push(&format!("/* TODO: dump struct init {expr:?} */"));
        },
        // TODO: dump dotfish
        Expr::Field { lhs, fields, .. } => {
            dump_expr(lhs, lines, session, max_len);

            for field in fields.iter() {
                lines.push(".");

                match field {
                    Field::Name { name, .. } => {
                        lines.push(&name.unintern_or_default(&session.intermediate_dir));
                    },
                    _ => todo!(),
                }
            }
        },
        Expr::FieldUpdate { lhs, fields, rhs } => {
            dump_expr(lhs, lines, session, max_len);

            for field in fields.iter() {
                lines.push(" `");
                lines.push(&field.unwrap_name().unintern_or_default(&session.intermediate_dir));
            }

            lines.push(" ");
            dump_expr(rhs, lines, session, max_len);
        },
        Expr::PrefixOp { op, rhs, .. } => {
            lines.push(op.render_error());
            dump_expr(rhs, lines, session, max_len);
        },
        Expr::InfixOp { lhs, op, rhs, .. } => {
            dump_expr(lhs, lines, session, max_len);

            match op {
                InfixOp::Index => {
                    lines.push("[");
                    dump_expr(rhs, lines, session, max_len);
                    lines.push("]");
                },
                _ => {
                    lines.push(op.render_error());
                    dump_expr(rhs, lines, session, max_len);
                },
            }
        },
        Expr::PostfixOp { lhs, op, .. } => {
            dump_expr(lhs, lines, session, max_len);
            lines.push(op.render_error());
        },
        Expr::TypeConversion { lhs, rhs, has_question_mark, .. } => {
            dump_expr(lhs, lines, session, max_len);
            lines.push(&format!(" as{} <", if *has_question_mark { "?" } else { "" }));
            dump_type(rhs, lines, session);
            lines.push(">");
        },
    }
}

fn lookahead_args(args: &[CallArg], session: &Session, max_len: usize) -> usize {
    let mut count = 0;

    for arg in args.iter() {
        let mut indented_lines = IndentedLines::new();
        dump_expr(&arg.arg, &mut indented_lines, session, max_len);
        count += indented_lines.dump().len();
    }

    count
}

fn lookahead_exprs(exprs: &[Expr], session: &Session, max_len: usize) -> usize {
    let mut count = 0;

    for expr in exprs.iter() {
        let mut indented_lines = IndentedLines::new();
        dump_expr(expr, &mut indented_lines, session, max_len);
        count += indented_lines.dump().len();
    }

    count
}

enum IfValueDumpType {
    Block,
    ShortExpr,
    LongExpr,
}

fn get_if_value_dump_type(value: &Expr, session: &Session) -> IfValueDumpType {
    match value {
        Expr::Block(_) => IfValueDumpType::Block,
        _ => {
            let mut indented_lines = IndentedLines::new();
            dump_expr(value, &mut indented_lines, session, 21);

            if indented_lines.dump().len() > 20 {
                IfValueDumpType::LongExpr
            } else {
                IfValueDumpType::ShortExpr
            }
        },
    }
}
