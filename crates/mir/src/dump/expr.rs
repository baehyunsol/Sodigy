use super::{dump_assert, dump_let};
use crate::{Callable, Expr, Session};
use sodigy_endec::IndentedLines;
use sodigy_hir::dump::dump_pattern;
use sodigy_parse::Field;

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
            if r#match.lowered_from_if {
                lines.push("/* lowered from if */");
            }

            lines.push("match ");
            dump_expr(&r#match.scrutinee, lines, session);
            lines.push(" {");
            lines.inc_indent();
            lines.break_line();

            for arm in r#match.arms.iter() {
                dump_pattern(&arm.pattern, lines, todo!());

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
            lines.push(" {");
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
        Expr::Call { func, args, .. } => {
            let (open_delim, close_delim) = match func {
                Callable::Static { def_span, .. } => {
                    lines.push(&session.span_to_string(*def_span).unwrap());
                    ("(", ")")
                },
                Callable::StructInit { .. } => todo!(),
                Callable::TupleInit { .. } => ("(", ")"),
                Callable::ListInit { .. } => ("[", "]"),
                Callable::Dynamic(f) => {
                    lines.push("(");
                    dump_expr(f, lines, session);
                    lines.push(")");
                    ("(", ")")
                },
            };

            lines.push(open_delim);

            if args.len() > 1 {
                lines.inc_indent();
                lines.break_line();

                for arg in args.iter() {
                    dump_expr(&arg, lines, session);
                    lines.push(",");
                    lines.break_line();
                }

                lines.dec_indent();
                lines.break_line();
            }

            else {
                for arg in args.iter() {
                    dump_expr(&arg, lines, session);
                }
            }

            lines.push(close_delim);
        },
    }
}
