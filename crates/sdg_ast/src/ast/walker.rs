use super::{AST, NameOrigin};
use crate::expr::ExprKind;
use crate::session::InternedString;
use crate::stmt::{ArgDef, Decorator};
use crate::value::{BlockDef, ValueKind};

impl AST {

    // it iterates all the exprs inside the AST
    // if it finds an ExprKind::Value(ValueKind::Identifier(name, origin)), it calls `f`
    pub fn id_walker(&self, f: impl Fn(&InternedString, &NameOrigin)) {

        for func in self.defs.values() {
            func.ret_val.kind.id_walker(&f);
            func.ret_type.as_ref().map(|t| t.kind.id_walker(&f));

            for Decorator { args, .. } in func.decorators.iter() {

                for arg in args.iter() {
                    arg.kind.id_walker(&f);
                }

            }

            for ArgDef { ty, .. } in func.args.iter() {
                ty.as_ref().map(|ty| ty.kind.id_walker(&f));
            }

        }

    }

}

impl ExprKind {

    pub fn id_walker(&self, f: &impl Fn(&InternedString, &NameOrigin)) {
        match self {
            ExprKind::Value(v) => match v {
                ValueKind::Integer(_)
                | ValueKind::Real(_)
                | ValueKind::String(_)
                | ValueKind::Bytes(_) => {},
                ValueKind::Identifier(name, origin) => f(name, origin),
                ValueKind::Format(elems)
                | ValueKind::List(elems)
                | ValueKind::Tuple(elems) => {
                    for elem in elems.iter() {
                        elem.kind.id_walker(f);
                    }
                },
                ValueKind::Lambda(_, _) => {
                    // call this function after all the lambdas are resolved
                    panic!("Internal Compiler Error 0E60BE5B42D");
                }
                ValueKind::Block { defs, value, .. } => {
                    for BlockDef { value, .. } in defs.iter() {
                        value.kind.id_walker(f);
                    }

                    value.kind.id_walker(f);
                }
            },
            ExprKind::Prefix(_, op) | ExprKind::Postfix(_, op) => {
                op.kind.id_walker(f);
            }
            ExprKind::Infix(_, op1, op2) => {
                op1.kind.id_walker(f);
                op2.kind.id_walker(f);
            }
            ExprKind::Call(func, args) => {
                func.kind.id_walker(f);

                for arg in args.iter() {
                    arg.kind.id_walker(f);
                }

            }
            ExprKind::Branch(c, t, fl) => {
                c.kind.id_walker(f);
                t.kind.id_walker(f);
                fl.kind.id_walker(f);
            }
        }
    }

}