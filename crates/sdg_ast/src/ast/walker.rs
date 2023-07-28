use super::{AST, NameOrigin};
use crate::expr::ExprKind;
use crate::session::InternedString;
use crate::stmt::{ArgDef, Decorator};
use crate::value::{BlockDef, ValueKind};

impl AST {

    // it iterates all the exprs inside the AST
    // if it finds an ExprKind::Value(ValueKind::Identifier(name, origin)), it calls `f`
    pub fn id_walker<Ctxt>(&self, f: impl Fn(&InternedString, &NameOrigin, &mut Ctxt), ctxt: &mut Ctxt) {

        for func in self.defs.values() {
            func.ret_val.kind.id_walker(&f, ctxt);
            func.ret_type.as_ref().map(|t| t.kind.id_walker(&f, ctxt));

            for Decorator { args, .. } in func.decorators.iter() {

                for arg in args.iter() {
                    arg.kind.id_walker(&f, ctxt);
                }

            }

            for ArgDef { ty, .. } in func.args.iter() {
                ty.as_ref().map(|ty| ty.kind.id_walker(&f, ctxt));
            }
        }

    }

}

impl ExprKind {

    pub fn id_walker<Ctxt>(&self, f: &impl Fn(&InternedString, &NameOrigin, &mut Ctxt), ctxt: &mut Ctxt) {
        match self {
            ExprKind::Value(v) => match v {
                ValueKind::Integer(_)
                | ValueKind::Real(_)
                | ValueKind::String(_)
                | ValueKind::Bytes(_) => {},
                ValueKind::Identifier(name, origin) => f(name, origin, ctxt),
                ValueKind::Format(elems)
                | ValueKind::List(elems)
                | ValueKind::Tuple(elems) => {
                    for elem in elems.iter() {
                        elem.kind.id_walker(f, ctxt);
                    }
                },
                ValueKind::Closure(_, captured_variables) => {
                    for (name, origin) in captured_variables.iter() {
                        f(name, origin, ctxt)
                    }
                },
                ValueKind::Lambda(_, _) => {
                    // call this function after all the lambdas are resolved
                    panic!("Internal Compiler Error 0E60BE5B42D");
                },
                ValueKind::Block { defs, value, .. } => {
                    for BlockDef { value, ty, .. } in defs.iter() {
                        value.kind.id_walker(f, ctxt);

                        if let Some(ty) = ty {
                            ty.kind.id_walker(f, ctxt);
                        }
                    }

                    value.kind.id_walker(f, ctxt);
                },
            },
            ExprKind::Prefix(_, op) | ExprKind::Postfix(_, op) => {
                op.kind.id_walker(f, ctxt);
            },
            ExprKind::Infix(_, op1, op2) => {
                op1.kind.id_walker(f, ctxt);
                op2.kind.id_walker(f, ctxt);
            },
            ExprKind::Call(func, args) => {
                func.kind.id_walker(f, ctxt);

                for arg in args.iter() {
                    arg.kind.id_walker(f, ctxt);
                }
            },

            // TODO: What do I do with patterns?
            ExprKind::Match(value, branches, _) => todo!(),

            ExprKind::Branch(c, t, fl) => {
                c.kind.id_walker(f, ctxt);
                t.kind.id_walker(f, ctxt);
                fl.kind.id_walker(f, ctxt);
            },
        }
    }

}