use crate::{FuncPurity, Session, Type};
use sodigy_endec::IndentedLines;
use sodigy_parse::Field;

pub fn dump_type(r#type: &Type, lines: &mut IndentedLines, session: &Session) {
    match r#type {
        Type::Ident(id) => {
            lines.push(&id.id.unintern_or_default(&session.intermediate_dir));
        },
        Type::Path { id, fields } => {
            lines.push(&id.id.unintern_or_default(&session.intermediate_dir));

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
        Type::Param { constructor, args, .. } => {
            dump_type(constructor, lines, session);
            lines.push("<");

            for arg in args.iter() {
                dump_type(arg, lines, session);
                lines.push(",");
            }

            lines.push(">");
        },
        Type::Tuple { types, .. } => {
            lines.push("(");

            if types.len() > 1 {
                lines.inc_indent();
                lines.break_line();

                for r#type in types.iter() {
                    dump_type(&r#type, lines, session);
                    lines.push(",");
                    lines.break_line();
                }

                lines.dec_indent();
                lines.break_line();
            }

            else {
                for r#type in types.iter() {
                    dump_type(&r#type, lines, session);
                }

                if types.len() == 1 {
                    lines.push(",");
                }
            }

            lines.push(")");
        },
        Type::Func { params, r#return, purity, .. } => {
            let f = match purity {
                FuncPurity::Both => "Fn",
                FuncPurity::Pure => "PureFn",
                FuncPurity::Impure => "ImpureFn",
            };
            lines.push(f);
            lines.push("(");

            for param in params.iter() {
                dump_type(param, lines, session);
                lines.push(",");
            }

            lines.push(") -> ");
            dump_type(r#return, lines, session);
        },
        Type::Wildcard(_) => {
            lines.push("_");
        },
        Type::Never(_) => {
            // TODO: maybe just use `Never` instead of introducing a new token...
            lines.push("!");
        },
    }
}
