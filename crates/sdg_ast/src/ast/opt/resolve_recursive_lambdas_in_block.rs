use super::super::{AST, ASTError, NameOrigin};
use super::substitute_local_def;
use crate::expr::{Expr, ExprKind, MatchBranch};
use crate::iter_mut_exprs_in_ast;
use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;
use crate::stmt::{ArgDef, Decorator, FuncKind};
use crate::value::{BlockDef, ValueKind};
use sdg_uid::UID;
use std::collections::HashMap;

// TODO: it complicates some unused name warnings!!

/*
```
{
    a = \{n, if n > 0 { a(n - 1) } else { 0 }};

    a
}
```

The name resolver thinks that `a` is a closure, because it's referencing `a`, which is not in the lambda's name scope.
But it's obvious that `a` is not a closure. This pass visits all the exprs, finds such cases, and fixes them.
It also deals with mutually recursive cases

1. if it finds `Call(@@LAMBDA_ABCDEF, a)`, which is a closure, it checks whether all of the arguments (captured vars) are functors
2. if so, it changes `Call(@@LAMBDA_ABCDEF, a)` to `@@LAMBDA_ABCDEF` and modify the def of `@@LAMBDA_ABCDEF` in AST

This pass must be called after name_resolve and before block_clean_up because,
1. name_resolve creates lambda definitions
2. block_clean_up will reject recursive lambda functions (without this pass) because they reject recursive block defs
*/

/*
 * In this file, `lambda` is an anonymous function without any captured variable
 * `closure` is an anonymous function with at least one captured variable
 */

iter_mut_exprs_in_ast!(resolve_recursive_lambdas_in_block, ClosureCollector);

impl Expr {
    pub fn resolve_recursive_lambdas_in_block(&mut self, session: &mut LocalParseSession, ctxt: &mut ClosureCollector) -> Result<(), ASTError> {

        match &mut self.kind {
            ExprKind::Value(v) => match v {
                ValueKind::Identifier(_, _)
                | ValueKind::Integer(_)
                | ValueKind::Real(_)
                | ValueKind::String(_)
                | ValueKind::Char(_)
                | ValueKind::Bytes(_)
                | ValueKind::Object(_) => {},
                ValueKind::List(elements)
                | ValueKind::Tuple(elements)
                | ValueKind::Format(elements) => {
                    for element in elements.iter_mut() {
                        element.resolve_recursive_lambdas_in_block(session, ctxt)?;
                    }
                },
                ValueKind::Closure(f, captured_variables) => {
                    let mut captured_names = Vec::with_capacity(captured_variables.len());

                    for var in captured_variables.iter() {
                        match var.kind {
                            ExprKind::Value(ValueKind::Identifier(name, origin)) => {
                                captured_names.push((name, origin));
                            },
                            ExprKind::Value(ValueKind::Closure(name, _)) => {
                                captured_names.push((name, NameOrigin::AnonymousFunc));
                            },
                            _ => {}
                        }
                    }

                    ctxt.collect_closure(*f, captured_names);
                },
                ValueKind::Lambda(args, val) => {
                    for ArgDef { ty, .. } in args.iter_mut() {
                        if let Some(ty) = ty {
                            ty.resolve_recursive_lambdas_in_block(session, ctxt)?;
                        }
                    }

                    val.resolve_recursive_lambdas_in_block(session, ctxt)?;
                },
                ValueKind::Block { defs, value, id } => {
                    ctxt.start_block(*id);

                    for BlockDef { value, ty, name, .. } in defs.iter_mut() {
                        value.resolve_recursive_lambdas_in_block(session, ctxt)?;

                        if let Some(ty) = ty {
                            ty.resolve_recursive_lambdas_in_block(session, ctxt)?;
                        }

                        // collect only when `value` is a closure or a lambda
                        ctxt.collect_block_value(value, *name);
                    }

                    if ctxt.solve_closures() {
                        for BlockDef { value, .. } in defs.iter_mut() {
                            value.replace_closure_with_lambda(&ctxt.closure_to_lambda_info);
                        }
                    }

                    value.resolve_recursive_lambdas_in_block(session, ctxt)?;

                    ctxt.finish_block();
                },
            },
            ExprKind::Prefix(_, v) => v.resolve_recursive_lambdas_in_block(session, ctxt)?,
            ExprKind::Postfix(_, v) => v.resolve_recursive_lambdas_in_block(session, ctxt)?,
            ExprKind::Infix(_, v1, v2) => {
                v1.resolve_recursive_lambdas_in_block(session, ctxt)?;
                v2.resolve_recursive_lambdas_in_block(session, ctxt)?;
            },
            ExprKind::Match(value, branches, _) => {
                value.resolve_recursive_lambdas_in_block(session, ctxt)?;

                for MatchBranch { value, .. } in branches.iter_mut() {
                    value.resolve_recursive_lambdas_in_block(session, ctxt)?;
                }
            },
            ExprKind::Branch(c, t, f) => {
                c.resolve_recursive_lambdas_in_block(session, ctxt)?;
                t.resolve_recursive_lambdas_in_block(session, ctxt)?;
                f.resolve_recursive_lambdas_in_block(session, ctxt)?;
            },
            ExprKind::Call(f, args) => {
                f.resolve_recursive_lambdas_in_block(session, ctxt)?;

                for arg in args.iter_mut() {
                    arg.resolve_recursive_lambdas_in_block(session, ctxt)?;
                }
            }
        }

        // though it doesn't return any error, it's return type is `Result`, due to the macro
        Ok(())
    }

    fn replace_closure_with_lambda(&mut self, ctxt: &ClosureToLambdaInfo) {
        match &mut self.kind {
            ExprKind::Value(v) => match v {
                ValueKind::Integer(_)
                | ValueKind::Real(_)
                | ValueKind::String(_)
                | ValueKind::Char(_)
                | ValueKind::Bytes(_)
                | ValueKind::Identifier(_, _)
                | ValueKind::Object(_) => {},
                ValueKind::Format(elems)
                | ValueKind::List(elems)
                | ValueKind::Tuple(elems) => {
                    for elem in elems.iter_mut() {
                        elem.replace_closure_with_lambda(ctxt);
                    }
                },
                ValueKind::Closure(name, _) => {
                    if ctxt.contains_key(name) {
                        self.kind = ExprKind::Value(ValueKind::Identifier(*name, NameOrigin::AnonymousFunc));
                    }
                },
                ValueKind::Lambda(_, val) => {
                    val.replace_closure_with_lambda(ctxt);
                },
                ValueKind::Block { defs, value, .. } => {
                    for BlockDef { value, ty, .. } in defs.iter_mut() {
                        value.replace_closure_with_lambda(ctxt);

                        if let Some(ty) = ty {
                            ty.replace_closure_with_lambda(ctxt);
                        }
                    }

                    value.replace_closure_with_lambda(ctxt);
                },
            },
            ExprKind::Prefix(_, op) | ExprKind::Postfix(_, op) => {
                op.replace_closure_with_lambda(ctxt);
            },
            ExprKind::Infix(_, op1, op2) => {
                op1.replace_closure_with_lambda(ctxt);
                op2.replace_closure_with_lambda(ctxt);
            },
            ExprKind::Call(func, args) => {
                func.replace_closure_with_lambda(ctxt);

                for arg in args.iter_mut() {
                    arg.replace_closure_with_lambda(ctxt);
                }
            },
            ExprKind::Match(value, branches, _) => {
                value.replace_closure_with_lambda(ctxt);

                for MatchBranch { value, .. } in branches.iter_mut() {
                    value.replace_closure_with_lambda(ctxt);
                }
            },

            ExprKind::Branch(c, t, fl) => {
                c.replace_closure_with_lambda(ctxt);
                t.replace_closure_with_lambda(ctxt);
                fl.replace_closure_with_lambda(ctxt);
            },
        }
    }
}

impl AST {
    pub fn modify_closure_defs(&mut self, closure_collector: &ClosureToLambdaInfo) {

        for (closure_name, captured_vars) in closure_collector.iter() {
            let substitutions = captured_vars.iter().map(
                |(name, closure_name)| (
                    name.clone(),
                    Expr::new_identifier(*closure_name, NameOrigin::AnonymousFunc, Span::dummy()),
                )
            ).collect();

            match self.defs.get_mut(closure_name) {
                Some(closure_def) => {
                    substitute_local_def(&mut closure_def.ret_val, &substitutions);
                    closure_def.kind = FuncKind::Lambda;
                },
                _ => unreachable!(
                    "Internal Compiler Error B7F15A8FD91"
                ),
            }
        }

    }
}

type ClosureName = InternedString;

// HashMap<ClosureName, HashMap<CapturedVarInfo, LambdaFuncThatTheCapturedVarActuallyPointsTo>>
type ClosureToLambdaInfo = HashMap<ClosureName, HashMap<(InternedString, NameOrigin), ClosureName>>;

#[derive(Clone)]
pub enum SolveState {
    DontKnow,
    NeverLambda,
    MustBeLambda,
    DependOn(Vec<ClosureName>),
}

enum BlockValue {
    WhoCares,
    Closure(ClosureName),
    Lambda(ClosureName),
}

pub struct ClosureCollector {
    block_stack: Vec<UID>,
    block_values: HashMap<UID, HashMap<InternedString, BlockValue>>,
    closures_by_block: HashMap<UID, Vec<ClosureName>>,
    curr_closure_solve_state: HashMap<ClosureName, SolveState>,
    curr_closures: HashMap<ClosureName, Vec<(InternedString, NameOrigin)>>,

    // this info is later used to modify the ast of lambda defs
    pub closure_to_lambda_info: ClosureToLambdaInfo,
}

impl ClosureCollector {
    pub fn new() -> Self {
        ClosureCollector {
            block_stack: vec![],
            block_values: HashMap::new(),
            closures_by_block: HashMap::new(),
            curr_closure_solve_state: HashMap::new(),
            curr_closures: HashMap::new(),
            closure_to_lambda_info: HashMap::new(),
        }
    }

    fn get_curr_block_id(&self) -> UID {
        self.block_stack.last().map(|id| *id).unwrap_or(UID::dummy())
    }

    pub fn start_block(&mut self, id: UID) {
        self.block_stack.push(id);
    }

    pub fn solve_closures(&mut self) -> bool {
        let curr_block_id = self.get_curr_block_id();

        // closures that are not in a block expression cannot be a lambda
        if curr_block_id.is_dummy() {
            return false;
        }

        let closures_to_solve = match self.closures_by_block.get(&curr_block_id) {
            Some(v) => v.iter().map(|name| *name).collect::<Vec<_>>(),
            _ => {
                return false;  // no closures in this block
            }
        };
        let mut has_something_to_do = false;

        for closure in closures_to_solve.into_iter() {
            has_something_to_do |= solve_closure_impl(closure, self);
        }

        has_something_to_do
    }

    pub fn set_solve_state(&mut self, name: ClosureName, state: SolveState) {
        match self.curr_closure_solve_state.get_mut(&name) {
            Some(s) => {
                if let SolveState::NeverLambda = state {
                    self.closure_to_lambda_info.remove(&name);
                }

                *s = state;
            }
            _ => {}
        }
    }

    pub fn get_solve_state(&self, name: ClosureName) -> SolveState {
        match self.curr_closure_solve_state.get(&name) {
            Some(s) => s.clone(),
            _ => unreachable!("Internal Compiler Error 6D8CD40DAC7"),
        }
    }

    pub fn set_closure_captured_var_info(&mut self, closure_name: ClosureName, captured_var: (InternedString, NameOrigin), lambda_name: ClosureName) {
        match self.closure_to_lambda_info.get_mut(&closure_name) {
            Some(vars) => {
                vars.insert(captured_var, lambda_name);
            },
            None => {
                let mut table = HashMap::new();
                table.insert(captured_var, lambda_name);

                self.closure_to_lambda_info.insert(closure_name, table);
            },
        }
    }

    pub fn collect_closure(&mut self, name: ClosureName, captured_vars: Vec<(InternedString, NameOrigin)>) {
        let curr_block_id = self.get_curr_block_id();

        match self.closures_by_block.get_mut(&curr_block_id) {
            Some(h) => {
                h.push(name);
            }
            None => {
                self.closures_by_block.insert(curr_block_id, vec![name]);
            }
        }

        self.closure_to_lambda_info.insert(name, HashMap::new());
        self.curr_closures.insert(name, captured_vars);
        self.curr_closure_solve_state.insert(name, SolveState::DontKnow);
    }

    pub fn collect_block_value(&mut self, val: &Expr, name: InternedString) {
        let curr_block_id = self.get_curr_block_id();

        if val.is_closure() || val.is_lambda() {
            let self_val = if val.is_closure() {
                BlockValue::Closure(val.unwrap_closure_name())
            } else {
                BlockValue::Lambda(val.unwrap_lambda_name())
            };

            match self.block_values.get_mut(&curr_block_id) {
                Some(h) => {
                    h.insert(name, self_val);
                },
                None => {
                    let mut curr = HashMap::new();

                    curr.insert(name, self_val);
                    self.block_values.insert(curr_block_id, curr);
                }
            }
        }
    }

    pub fn finish_block(&mut self) {
        let curr_block_id = self.get_curr_block_id();
        if let Some(closures_to_remove) = self.closures_by_block.get(&curr_block_id) {
            for closure in closures_to_remove.iter() {
                self.curr_closure_solve_state.remove(&closure);
                self.curr_closures.remove(&closure);
            }

            self.closures_by_block.remove(&curr_block_id);
        }

        self.block_stack.pop().expect("Internal Compiler Error DDAB28E412D");
    }
}

/*
 * There are 3 types of variables that a closure can capture
 * A. another closure
 * B. a lambda function
 * C. others
 *
 * If a closure has 1 or more `C`, it's a closure.
 * If a closure doesn't have any `C`, it has to look at all the `A`s it has.
 * If all the `A`s that it has have no `C`s in their captured context, this function is a lambda.
 */
// returns true if it's a lambda
fn solve_closure_impl(closure: ClosureName, ctxt: &mut ClosureCollector) -> bool {

    match ctxt.get_solve_state(closure) {
        SolveState::DontKnow => {
            let mut dependencies = vec![];
            let curr_context = ctxt.curr_closures.get(&closure).expect("
                Internal Compiler Error C6F26CA71D3
            ").clone();

            for (var_name, var_origin) in curr_context.into_iter() {
                match var_origin {
                    NameOrigin::BlockDef(id) => {
                        match ctxt.block_values.get(&id).expect(
                            "Internal Compiler Error 5493D3F0E85"
                        ).get(&var_name).expect(
                            "Internal Compiler Error F2BE56D9815"
                        ) {
                            // e.g. captures a local variable
                            BlockValue::WhoCares => {
                                ctxt.set_solve_state(closure, SolveState::NeverLambda);
                                return false;
                            }
                            BlockValue::Closure(child) => {
                                let child = *child;
                                // set state temporarily in order to prevent infinite recursion
                                ctxt.set_solve_state(closure, SolveState::DependOn(vec![]));

                                if !solve_closure_impl(child, ctxt) {
                                    ctxt.set_solve_state(closure, SolveState::NeverLambda);
                                    return false;
                                }

                                ctxt.set_closure_captured_var_info(
                                    closure,
                                    (var_name, var_origin),
                                    child,
                                );

                                dependencies.push(child);
                            }
                            BlockValue::Lambda(lambda_name) => {
                                ctxt.set_closure_captured_var_info(
                                    closure,
                                    (var_name, var_origin),
                                    *lambda_name,
                                );
                                continue;
                            }
                        }
                    },
                    _ => {
                        ctxt.set_solve_state(closure, SolveState::NeverLambda);
                        return false;
                    }
                }
            }

            if dependencies.is_empty() {
                ctxt.set_solve_state(closure, SolveState::MustBeLambda);

                true
            } else {
                ctxt.set_solve_state(closure, SolveState::DependOn(dependencies.clone()));

                for dependency in dependencies.iter() {
                    match ctxt.get_solve_state(*dependency) {
                        SolveState::NeverLambda => { return false; },
                        _ => {}
                    }
                }

                true
            }
        },
        SolveState::NeverLambda => false,
        SolveState::MustBeLambda => true,
        SolveState::DependOn(dependencies) => {
            let mut dependencies = dependencies.clone();

            while let Some(dependency) = dependencies.pop() {
                match ctxt.get_solve_state(dependency) {
                    SolveState::NeverLambda => {
                        ctxt.set_solve_state(closure, SolveState::NeverLambda);

                        return false;
                    },
                    SolveState::DontKnow => {
                        solve_closure_impl(dependency, ctxt);
                        dependencies.push(dependency);
                    }
                    _ => {}
                }
            }

            true
        },
    }
}
