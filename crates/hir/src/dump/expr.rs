use super::{dump_assert, dump_let, dump_pattern};
use crate::{Expr, Session};
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

            dump_expr(&r#if.cond, lines, session);
            lines.push(" {");
            lines.inc_indent();
            lines.break_line();
            dump_expr(&r#if.true_value, lines, session);
            lines.dec_indent();
            lines.break_line();
            lines.push("} else {");
            lines.inc_indent();
            lines.break_line();
            dump_expr(&r#if.false_value, lines, session);
            lines.dec_indent();
            lines.break_line();
            lines.push("}");
        },
        Expr::Match(r#match) => {
            lines.push("match ");
            dump_expr(&r#match.scrutinee, lines, session);
            lines.push(" {");
            lines.inc_indent();
            lines.break_line();

            for arm in r#match.arms.iter() {
                dump_pattern(&arm.pattern, lines, session);

                if let Some(guard) = &arm.guard {
                    lines.push(" if ");
                    dump_expr(guard, lines, session);
                }

                lines.push(" => ");
                dump_expr(&arm.value, lines, session);
                lines.push(",");
                lines.break_line();
            }

            lines.dec_indent();
            lines.break_line();
            lines.push("}");
        },
        Expr::Block(block) => {
            lines.push("{");
            lines.inc_indent();
            lines.break_line();

            for r#let in block.lets.iter() {
                dump_let(r#let, lines, session);
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

            if args.len() > 1 {
                lines.inc_indent();
                lines.break_line();

                for arg in args.iter() {
                    dump_expr(&arg.arg, lines, session);
                    lines.push(",");
                    lines.break_line();
                }

                lines.dec_indent();
                lines.break_line();
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

            if elements.len() > 1 {
                lines.inc_indent();
                lines.break_line();

                for element in elements.iter() {
                    dump_expr(&element, lines, session);
                    lines.push(",");
                    lines.break_line();
                }

                lines.dec_indent();
                lines.break_line();
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
