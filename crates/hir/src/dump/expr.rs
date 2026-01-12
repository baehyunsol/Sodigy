use super::{dump_assert, dump_let, dump_pattern};
use crate::{CallArg, Expr, Session};
use sodigy_endec::IndentedLines;
use sodigy_parse::Field;
use sodigy_token::InfixOp;

pub fn dump_expr(expr: &Expr, lines: &mut IndentedLines, session: &Session) {
    match expr {
        Expr::Ident(id) => {
            lines.push(&id.id.unintern_or_default(&session.intermediate_dir));
        },
        Expr::Number { n, .. } => {
            lines.push(&n.dump());
        },
        Expr::String { binary, s, .. } => {
            let s = format!(
                "{}{:?}",
                if *binary { "b" } else { "" },
                s.unintern_or_default(&session.intermediate_dir),
            );
            lines.push(&s);
        },
        Expr::Char { ch, .. } => {
            lines.push(&format!("{:?}", char::from_u32(*ch).unwrap()));
        },
        Expr::Byte { b, .. } => {
            lines.push(&format!("#{b}"));
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

            dump_expr(&r#if.cond, lines, session);

            if if_cond {
                lines.push(")");
            }

            match get_if_value_dump_type(&r#if.true_value, session) {
                IfValueDumpType::Block => {
                    lines.push(" ");
                    dump_expr(&r#if.true_value, lines, session);
                    lines.push(" else");
                },
                IfValueDumpType::ShortExpr => {
                    lines.push(" { ");
                    dump_expr(&r#if.true_value, lines, session);
                    lines.push(" } else");
                },
                IfValueDumpType::LongExpr => {
                    lines.push(" {");
                    lines.inc_indent();
                    lines.break_line();
                    dump_expr(&r#if.true_value, lines, session);
                    lines.dec_indent();
                    lines.break_line();
                    lines.push("} else");
                },
            }

            match get_if_value_dump_type(&r#if.false_value, session) {
                IfValueDumpType::Block => {
                    lines.push(" ");
                    dump_expr(&r#if.false_value, lines, session);
                },
                IfValueDumpType::ShortExpr => {
                    lines.push(" { ");
                    dump_expr(&r#if.false_value, lines, session);
                    lines.push(" }");
                },
                IfValueDumpType::LongExpr => {
                    lines.push(" {");
                    lines.inc_indent();
                    lines.break_line();
                    dump_expr(&r#if.false_value, lines, session);
                    lines.dec_indent();
                    lines.break_line();
                    lines.push("}");
                },
            }
        },
        Expr::Match(r#match) => {
            lines.push("match ");
            dump_expr(&r#match.scrutinee, lines, session);
            lines.push(" {");
            lines.inc_indent();
            lines.break_line();

            for (i, arm) in r#match.arms.iter().enumerate() {
                dump_pattern(&arm.pattern, lines, session);

                if let Some(guard) = &arm.guard {
                    lines.push(" if ");
                    dump_expr(guard, lines, session);
                }

                lines.push(" => ");
                dump_expr(&arm.value, lines, session);
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

            dump_expr(&block.value, lines, session);
            lines.dec_indent();
            lines.break_line();
            lines.push("}");
        },
        Expr::Call { func, args, .. } => {
            match &**func {
                Expr::Ident(_) | Expr::Path { .. } => {
                    dump_expr(func, lines, session);
                },
                _ => {
                    lines.push("(");
                    dump_expr(func, lines, session);
                    lines.push(")");
                },
            }

            lines.push("(");
            let arg_per_line = lookahead_args(&args, session) > 20;

            if args.len() > 1 {
                if arg_per_line {
                    lines.inc_indent();
                    lines.break_line();
                }

                for (i, arg) in args.iter().enumerate() {
                    dump_expr(&arg.arg, lines, session);
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
                    dump_expr(&arg.arg, lines, session);
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
            let element_per_line = lookahead_exprs(elements, session) > 20;

            if elements.len() > 1 {
                if element_per_line {
                    lines.inc_indent();
                    lines.break_line();
                }

                for (i, element) in elements.iter().enumerate() {
                    dump_expr(&element, lines, session);
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
                    dump_expr(&element, lines, session);
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
        Expr::Path { lhs, fields } => {
            dump_expr(lhs, lines, session);

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
        Expr::FieldModifier { lhs, fields, rhs } => {
            dump_expr(lhs, lines, session);

            for (field, _) in fields.iter() {
                lines.push(" `");
                lines.push(&field.unintern_or_default(&session.intermediate_dir));
            }

            lines.push(" ");
            dump_expr(rhs, lines, session);
        },
        Expr::PrefixOp { op, rhs, .. } => {
            lines.push(op.render_error());
            dump_expr(rhs, lines, session);
        },
        Expr::InfixOp { lhs, op, rhs, .. } => {
            dump_expr(lhs, lines, session);

            match op {
                InfixOp::Index => {
                    lines.push("[");
                    dump_expr(rhs, lines, session);
                    lines.push("]");
                },
                _ => {
                    lines.push(op.render_error());
                    dump_expr(rhs, lines, session);
                },
            }
        },
        Expr::PostfixOp { .. } => todo!(),
    }
}

fn lookahead_args(args: &[CallArg], session: &Session) -> usize {
    let mut count = 0;

    for arg in args.iter() {
        let mut indented_lines = IndentedLines::new();
        dump_expr(&arg.arg, &mut indented_lines, session);
        count += indented_lines.dump().len();
    }

    count
}

fn lookahead_exprs(exprs: &[Expr], session: &Session) -> usize {
    let mut count = 0;

    for expr in exprs.iter() {
        let mut indented_lines = IndentedLines::new();
        dump_expr(&expr, &mut indented_lines, session);
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
            dump_expr(value, &mut indented_lines, session);

            if indented_lines.dump().len() > 20 {
                IfValueDumpType::LongExpr
            } else {
                IfValueDumpType::ShortExpr
            }
        },
    }
}
