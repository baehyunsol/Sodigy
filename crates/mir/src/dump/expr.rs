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
            if r#match.lowered_from_if {
                lines.push("/* lowered from if */");
            }

            lines.push("match ");
            dump_expr(&r#match.scrutinee, lines, session);
            lines.push(" {");
            lines.inc_indent();
            lines.break_line();

            for arm in r#match.arms.iter() {
                dump_pattern(&arm.pattern, lines, &into_hir_session(session));

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
        Expr::Path { lhs, fields } => {
            dump_expr(lhs, lines, session);

            for field in fields.iter() {
                lines.push(".");

                match field {
                    Field::Name { name, .. } => {
                        lines.push(&name.unintern_or_default(&session.intermediate_dir));
                    },
                    Field::Index(i) => {
                        if *i < 0 { todo!() }
                        lines.push(&format!("_{i}"));
                    },
                    Field::Range(_, _) => todo!(),
                    Field::Variant => {
                        lines.push("__VARIANT__");
                    },
                    Field::Constructor => {
                        lines.push("__CONSTRUCTOR__");
                    },
                    Field::Payload => {
                        lines.push("__PAYLOAD__");
                    },
                }
            }
        },
        Expr::FieldModifier { lhs, fields, rhs } => {
            dump_expr(lhs, lines, session);

            for field in fields.iter() {
                lines.push(" `");
                lines.push(&field.unwrap_name().unintern_or_default(&session.intermediate_dir));
            }

            lines.push(" ");
            dump_expr(rhs, lines, session);
        },
        Expr::Call { func, args, .. } => {
            let (open_delim, close_delim) = match func {
                Callable::Static { def_span, .. } => {
                    lines.push(&session.span_to_string(*def_span).unwrap_or_else(|| format!("({def_span:?})")));
                    ("(", ")")
                },
                Callable::StructInit { def_span, .. } => {
                    lines.push(&session.span_to_string(*def_span).unwrap_or_else(|| format!("({def_span:?})")));
                    ("{", "}")
                },
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
            let arg_per_line = lookahead_args(&args, session) > 20;

            if args.len() > 1 {
                if arg_per_line {
                    lines.inc_indent();
                    lines.break_line();
                }

                for (i, arg) in args.iter().enumerate() {
                    dump_expr(&arg, lines, session);
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
                    dump_expr(&arg, lines, session);
                }
            }

            lines.push(close_delim);
        },
    }
}

fn lookahead_args(args: &[Expr], session: &Session) -> usize {
    let mut count = 0;

    for arg in args.iter() {
        let mut indented_lines = IndentedLines::new();
        dump_expr(arg, &mut indented_lines, session);
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

use std::collections::HashMap;

// TODO: In order to `dump_type`, I need an HirSession, but I don't have one.
//       This is just a quick hack. I don't like it and I have to find a better way.
fn into_hir_session(session: &Session) -> sodigy_hir::Session {
    sodigy_hir::Session {
        intermediate_dir: session.intermediate_dir.to_string(),
        name_stack: vec![],
        attribute_rule_cache: HashMap::new(),
        func_default_values: vec![],
        is_in_debug_context: false,
        is_std: false,
        nested_pipeline_depth: 0,
        lets: vec![],
        funcs: vec![],
        structs: vec![],
        enums: vec![],
        asserts: vec![],
        aliases: vec![],
        uses: vec![],
        modules: vec![],
        type_assertions: vec![],
        associated_items: vec![],
        lang_items: HashMap::new(),
        polys: HashMap::new(),
        poly_impls: vec![],
        errors: vec![],
        warnings: vec![],
    }
}
