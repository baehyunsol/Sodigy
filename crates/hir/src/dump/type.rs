use crate::{Session, Type};
use sodigy_endec::IndentedLines;

pub fn dump_type(r#type: &Type, lines: &mut IndentedLines, session: &Session) {
    match r#type {
        Type::Path(path) => {
            lines.push(&path.unintern_or_default(&session.intermediate_dir));
        },
        Type::Param { constructor, args, .. } => {
            lines.push(&constructor.unintern_or_default(&session.intermediate_dir));
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
        Type::Func { fn_constructor, params, r#return, .. } => {
            lines.push(&fn_constructor.unintern_or_default(&session.intermediate_dir));
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
