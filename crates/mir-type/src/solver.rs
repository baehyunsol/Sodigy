use crate::TypeError;
use crate::preludes::*;
use sodigy_mir::{Expr, Type};
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

pub struct Solver {
    pub preludes: Vec<InternedString>,
}

impl Solver {
    pub fn new() -> Self {
        Solver {
            preludes: get_preludes(),
        }
    }

    // FIXME: there are A LOT OF heap allocations
    //
    // It can solve type of any expression, but the result maybe `Type::Var`.
    // If it finds new type equations while solving, it adds them to `type_equations`.
    //
    // It's a type-inferer, not a type-checker.
    // If it sees a type equation like `Bool = Int`, it just ignores.
    // But if it sees a type equation like `Var(1) = Bool; Var(1) = Int`, it throws an error.
    //
    // The type-checker may throw multiple errors, but the inferer can throw at most 1 error,
    // because an erroneous inference can easily generate hard-to-understand errors.
    // And that's why the type-inferer tries to ignore errors -> so that the compiler can
    // find more errors.
    pub fn infer_expr(
        &mut self,
        expr: &Expr,
        types: &mut HashMap<Span, Type>,
    ) -> Result<Type, TypeError> {
        match expr {
            Expr::Identifier(id) => match types.get(&id.def_span) {
                Some(Type::Var(def_span)) => Ok(Type::Var(*def_span)),
                Some(r#type) => Ok(r#type.clone()),
                None => Ok(Type::Var(id.def_span)),
            },
            Expr::Number { n, .. } => match n.is_integer {
                true => Ok(Type::Static(Span::Prelude(self.preludes[INT]))),
                false => Ok(Type::Static(Span::Prelude(self.preludes[NUMBER]))),
            },
            Expr::String { binary, .. } => match *binary {
                true => Ok(Type::Static(Span::Prelude(self.preludes[BYTES]))),
                false => Ok(Type::Static(Span::Prelude(self.preludes[STRING]))),
            },
            Expr::If(r#if) => {
                let cond_type = self.infer_expr(r#if.cond.as_ref(), types)?;

                match cond_type {
                    Type::Static(Span::Prelude(s)) if s == self.preludes[BOOL] => {},  // okay
                    _ => {
                        if cond_type.has_variable() {
                            self.unify(
                                &cond_type,
                                &Type::Static(Span::Prelude(self.preludes[BOOL])),
                                types,
                            )?;
                        }
                    },
                }

                let true_type = self.infer_expr(r#if.true_value.as_ref(), types)?;
                let false_type = self.infer_expr(r#if.false_value.as_ref(), types)?;

                if true_type.has_variable() {
                    self.unify(
                        &true_type,
                        &false_type,
                        types,
                    )?;
                }

                else if false_type.has_variable() {
                    self.unify(
                        &false_type,
                        &true_type,
                        types,
                    )?;
                }

                Ok(true_type)
            },
            Expr::Call { func, args } => todo!(),
            _ => todo!(),
        }
    }

    // If it finds new information from the equation, it updates `self` and `types`.
    // If it finds 모순 while unifying, it returns an error
    pub fn unify(&mut self, lhs: &Type, rhs: &Type, types: &mut HashMap<Span, Type>) -> Result<(), TypeError> {
        todo!()
    }
}

trait TypeSolve {
    fn has_variable(&self) -> bool;
}

impl TypeSolve for Type {
    fn has_variable(&self) -> bool {
        match self {
            Type::Static(_) |
            Type::GenericDef(_) |
            Type::Unit(_) => false,
            Type::Generic {
                r#type,
                args,
                ..
            } => {
                r#type.has_variable() ||
                args.iter().any(|arg| arg.has_variable())
            },
            Type::Var(_) => true,
        }
    }
}
