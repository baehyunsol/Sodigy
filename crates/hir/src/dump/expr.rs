use crate::{Expr, Session};
use sodigy_endec::IndentedLines;

pub fn dump_expr(expr: &Expr, lines: &mut IndentedLines, session: &Session) {
    match expr {
        Expr::Ident(id) => {
            lines.push(&id.id.unintern_or_default(&session.intermediate_dir));
        },
        Expr::Number { n, .. } => {
            lines.push(&n.render());
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
                dump_let(assert, lines, session);
            }

            dump_expr(&block.value, lines, session);
            lines.dec_indent();
            lines.break_line();
        },
        Expr::Call { func, args, .. } => {
            match func {
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

                for arg in args.len() {
                    dump_expr(arg, lines, session);
                    lines.push(",");
                    lines.break_line();
                }

                lines.dec_indent();
                lines.break_line();
            }

            else {
                for arg in args.len() {
                    dump_expr(arg, lines, session);
                }
            }

            lines.push(")");
        },
    }
}
