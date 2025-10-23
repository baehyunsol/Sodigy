use crate::Type;
use crate::error::{ErrorContext, TypeError, TypeErrorKind};
use crate::preludes::*;
use sodigy_span::Span;
use sodigy_string::InternedString;
use sodigy_token::InfixOp;
use std::collections::hash_map::{Entry, HashMap};

mod assert;
mod expr;
mod func;
mod r#let;

pub struct Solver {
    // Whenever `types.get(span)` returns `None`, it creates a type variable
    // and inserts the `span` to this hash set. It's later used to check
    // if all the type variables are infered.
    // If the type variable is from a type annotation and a name is bound to
    // the type annotation, it also collects the name: that'd be helpful when
    // creating error messages.
    pub type_vars: HashMap<Span, Option<InternedString>>,

    pub preludes: Vec<InternedString>,
    pub infix_op_type_signatures: HashMap<InfixOp, Vec<Vec<Type>>>,
    pub errors: Vec<TypeError>,
}

impl Solver {
    pub fn new() -> Self {
        let preludes = get_preludes();

        // TODO: better way to manage this list?
        let infix_op_type_signatures = vec![
            (
                InfixOp::Add,
                vec![
                    vec![
                        Type::Static(Span::Prelude(preludes[INT])),
                        Type::Static(Span::Prelude(preludes[INT])),
                        Type::Static(Span::Prelude(preludes[INT])),
                    ],
                ],
            ),
        ].into_iter().collect();

        Solver {
            type_vars: HashMap::new(),
            preludes,
            infix_op_type_signatures,
            errors: vec![],
        }
    }

    pub fn check_all_types_infered(&mut self, types: &HashMap<Span, Type>) {
        for (def_span, id) in self.type_vars.iter() {
            match types.get(def_span) {
                None | Some(Type::Var { .. }) => {
                    self.errors.push(TypeError {
                        kind: TypeErrorKind::CannotInferType { id: *id },
                        span: *def_span,
                        extra_span: None,
                        context: ErrorContext::None,
                    });
                },
                Some(t) => {
                    let type_vars = t.get_type_vars();

                    if !type_vars.is_empty() {
                        self.errors.push(TypeError {
                            kind: TypeErrorKind::PartiallyInferedType { id: *id, r#type: t.clone() },
                            span: *def_span,
                            extra_span: None,
                            context: ErrorContext::None,
                        });
                    }
                },
            }
        }
    }

    pub fn add_type_variable(&mut self, def_span: Span, id: Option<InternedString>) {
        match self.type_vars.entry(def_span) {
            Entry::Occupied(mut e) if id.is_some() => {
                *e.get_mut() = id;
            },
            Entry::Vacant(e) => {
                e.insert(id);
            },
            _ => {},
        }
    }

    // It first checks whether `lhs` and `rhs` are equal. There's no subtyping in Sodigy.
    // If either operand is a type variable, it gets a new type equation.
    // It tries to solve the type equation. If it solves, it updates `types` and `self.`
    // If it finds non-sense while solving, it pushes the error to `self.errors` and returns `Err(())`.
    pub fn equal(
        &mut self,

        // It's a type equation `lhs == rhs`
        // If there's an error, the message would be "expected {lhs}, got {rhs}".
        lhs: &Type,
        rhs: &Type,

        types: &mut HashMap<Span, Type>,

        // for helpful error messages
        span: Span,
        extra_span: Option<Span>,
        context: ErrorContext,
    ) -> Result<(), ()> {
        match (lhs, rhs) {
            (Type::Unit(_), Type::Unit(_)) => Ok(()),
            (Type::Static(lhs_def), Type::Static(rhs_def)) => {
                if *lhs_def == *rhs_def {
                    Ok(())
                }

                else {
                    self.errors.push(TypeError {
                        kind: TypeErrorKind::UnexpectedType {
                            expected: lhs.clone(),
                            got: rhs.clone(),
                        },
                        span,
                        extra_span,
                        context,
                    });
                    Err(())
                }
            },
            (
                Type::Var { def_span, is_return: false },
                concrete @ (Type::Static(_) | Type::GenericDef(_) | Type::Unit(_)),
            ) | (
                concrete @ (Type::Static(_) | Type::GenericDef(_) | Type::Unit(_)),
                Type::Var { def_span, is_return: false },
            ) => {
                types.insert(*def_span, concrete.clone());
                self.substitute(*def_span, concrete, types)
            },
            (
                Type::Var { def_span, is_return: true },
                concrete @ (Type::Static(_) | Type::GenericDef(_) | Type::Unit(_)),
            ) | (
                concrete @ (Type::Static(_) | Type::GenericDef(_) | Type::Unit(_)),
                Type::Var { def_span, is_return: true },
            ) => {
                match types.get_mut(def_span) {
                    Some(Type::Func { r#return, .. }) => {
                        *r#return = Box::new(concrete.clone());
                    },
                    _ => unreachable!(),
                }

                self.substitute(*def_span, concrete, types)
            },
            (
                Type::GenericDef(_),
                concrete @ (Type::Static(_) | Type::Unit(_)),
            ) | (
                concrete @ (Type::Static(_) | Type::Unit(_)),
                Type::GenericDef(_),
            ) => {
                self.errors.push(TypeError {
                    kind: TypeErrorKind::GenericIsNotGeneric {
                        got: concrete.clone(),
                    },
                    span,
                    extra_span,
                    context,
                });
                Err(())
            },
            _ => panic!("TODO: {:?}", (lhs, rhs)),
        }
    }

    fn substitute(&mut self, var: Span, r#type: &Type, types: &mut HashMap<Span, Type>) -> Result<(), ()> {
        // TODO: as of now, there's nothing to substitute (it doesn't store any type equations)
        Ok(())
    }

    fn get_possible_type_signatures(&self, op: InfixOp) -> &[Vec<Type>] {
        self.infix_op_type_signatures.get(&op).unwrap()
    }
}
