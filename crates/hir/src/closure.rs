use crate::{Expr, Func, Session, TrivialLet};
use sodigy_name_analysis::{NameKind, NameOrigin};
use sodigy_span::Span;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct CapturedNames {
    pub locals: Vec<Span>,
    pub globals: Vec<Span>,

    // Technically, it has to be treated like `.locals`, but the optimizer will
    // 100% substitute constants, so we can treat it like globals.
    pub constants: Vec<Span>,
}

impl Session {
    pub fn check_captured_names(&mut self, lambdas: &mut Vec<Func>) -> Vec<(Func, Option<CapturedNames>)> {
        let mut result = Vec::with_capacity(lambdas.len());

        for lambda in lambdas.drain(..) {
            // In `fn(x) = \(y) => x + y;`, there's nothing we can do with `x`.
            // We have to capture `x` and make a closure.
            let mut local_values = vec![];

            // `Int` in `\(x: Int) => x + 1` is a foreign name, but we don't have to capture it!
            let mut not_values = vec![];

            // If there's `\(x) => x + y;` and `y` is a global value (top-level `let`), we don't
            // have to capture `y` because it's always available!
            let mut global_values = vec![];

            // In `{ let x = 3; \(y) => x + y }`, `x` is a local value, but we dont' have to
            // capture `x` because it's a constant. We can lower the closure to `\(y) => 3 + y`.
            let mut constants = vec![];

            for (name, (origin, def_span)) in lambda.foreign_names.iter() {
                let (origin, def_span) = (*origin, *def_span);

                match origin {
                    NameOrigin::FuncParam { .. } => {
                        local_values.push(def_span);
                    },
                    NameOrigin::GenericParam { .. } => {
                        not_values.push(def_span);
                    },
                    NameOrigin::Local { kind } | NameOrigin::Foreign { kind } => match kind {
                        // `NameKind::Func` can be a closure, but that's okay.
                        // If it were a closure, the captured variables must be in `lambda.foreign_names`.
                        NameKind::Let { is_top_level: true } |
                        NameKind::Func |
                        NameKind::EnumVariant { .. } => {
                            global_values.push(def_span);
                        },
                        // we need further check
                        NameKind::Let { is_top_level: false } => match self.trivial_lets.get(&def_span) {
                            Some(TrivialLet::Constant(_)) | Some(TrivialLet::IsLambda(_)) => {
                                constants.push(def_span);
                            },
                            // It'd be nice to check `self.trivial_lets` recursively, but I'm just too lazy to
                            // implement that...
                            Some(TrivialLet::Reference(ref_def_span)) => match self.trivial_lets.get(ref_def_span) {
                                Some(TrivialLet::Constant(_)) => {
                                    constants.push(def_span);
                                },
                                _ => {
                                    local_values.push(def_span);
                                },
                            },
                            // I want to implement a heuristic here:
                            // `let f1 = \(x) => if x == 0 { 0 } else { 1 + f2(x - 1) };`
                            // `let f2 = \(x) => if x == 0 { 0 } else { 1 + f1(x - 1) };`
                            // -> I want the compiler to be smart enough to figure out that `f1` and `f2` are not closures.
                            Some(TrivialLet::MaybeLambda(_)) => todo!(),
                            None => {
                                local_values.push(def_span);
                            },
                        },
                        // inter-hir guarantees that `NameKind::Use` is not a local value
                        NameKind::Struct |
                        NameKind::Enum |
                        NameKind::Alias |
                        NameKind::Module |
                        NameKind::Use |
                        NameKind::GenericParam => {
                            not_values.push(def_span);
                        },
                        NameKind::FuncParam |
                        NameKind::PatternNameBind |
                        NameKind::Pipeline => {
                            local_values.push(def_span);
                        },
                    },
                    // inter-hir guarantees that `use` cannot alias a local value
                    NameOrigin::External => {
                        not_values.push(def_span);
                    },
                }
            }

            match (local_values.len(), constants.len(), global_values.len(), not_values.len()) {
                (1.., _, _, _) => {
                    result.push((lambda, Some(CapturedNames {
                        locals: local_values,
                        globals: global_values,
                        constants,
                    })));
                },
                _ => {
                    result.push((lambda, None));
                },
            }
        }

        result
    }

    pub fn substitute_closures(&mut self) {
        if self.closures.is_empty() {
            return;
        }

        let mut closures = self.closures.clone();

        for r#let in self.lets.iter_mut() {
            substitute_closures_recursive(&mut r#let.value, &mut closures);

            if closures.is_empty() {
                return;
            }
        }

        for func in self.funcs.iter_mut() {
            substitute_closures_recursive(&mut func.value, &mut closures);

            if closures.is_empty() {
                return;
            }
        }

        for assert in self.asserts.iter_mut() {
            if let Some(note) = &mut assert.note {
                substitute_closures_recursive(note, &mut closures);
            }

            substitute_closures_recursive(&mut assert.value, &mut closures);

            if closures.is_empty() {
                return;
            }
        }
    }
}

fn substitute_closures_recursive(value: &mut Expr, closures: &mut HashMap<Span, CapturedNames>) {
    match value {
        Expr::Path(p) if p.fields.is_empty() => match closures.get(&p.id.def_span) {
            Some(captures) => {
                let def_span = p.id.def_span;
                *value = Expr::Closure {
                    fp: p.clone(),
                    captures: captures.locals.clone(),
                };

                // each closure must be unique!
                closures.remove(&def_span);
            },
            None => {},
        },
        Expr::Constant(_) => {},
        _ => todo!(),
    }
}
