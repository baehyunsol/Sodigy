use super::{Expr, ExprKind};
use crate::ast::{ASTError, NameOrigin, NameScope, NameScopeKind};
use crate::expr::InfixOp;
use crate::session::{InternedString, LocalParseSession};
use crate::stmt::{ArgDef, FuncDef, FuncKind};
use crate::value::{BlockDef, ValueKind};
use sdg_uid::UID;
use std::collections::{HashMap, HashSet};

impl Expr {
    pub fn resolve_names(
        &mut self,
        name_scope: &mut NameScope,
        lambda_defs: &mut HashMap<InternedString, FuncDef>,
        session: &mut LocalParseSession,
        used_names: &mut HashSet<(InternedString, NameOrigin)>,
    ) -> Result<(), ASTError> {
        match &mut self.kind {
            ExprKind::Value(v) => match v {
                ValueKind::Integer(_)
                | ValueKind::Real(_)
                | ValueKind::String(_)
                | ValueKind::Bytes(_) => {
                    Ok(())
                },
                ValueKind::List(elements)
                | ValueKind::Tuple(elements)
                | ValueKind::Format(elements) => {
                    for elem in elements.iter_mut() {
                        elem.resolve_names(name_scope, lambda_defs, session, used_names)?;
                    }

                    Ok(())
                },
                ValueKind::Identifier(id, _) => match name_scope.search_name(*id) {
                    Ok((None, origin)) => {
                        used_names.insert((*id, origin));
                        self.kind.set_origin(origin);

                        Ok(())
                    },
                    Ok((Some(alias), _)) => {
                        // its origin is handled by `.to_path`
                        self.kind = alias.to_path();

                        Ok(())
                    },
                    Err(()) => Err(ASTError::no_def(*id, self.span, name_scope.clone())),
                },
                ValueKind::Closure(_, _) => {
                    // not generated yet
                    unreachable!("Internal Compiler Error 34C5DBB43F6");
                },
                ValueKind::Lambda(args, expr) => {
                    let lambda_id = UID::new_lambda_id();

                    // TODO: `name_scope.push_names` after `ty.resolve_names`?
                    // -> dependent types?
                    name_scope.push_names(args, NameScopeKind::LambdaArg(lambda_id));

                    for ArgDef { ty, .. } in args.iter_mut() {
                        if let Some(ty) = ty {
                            ty.resolve_names(name_scope, lambda_defs, session, used_names)?;
                        }
                    }

                    expr.resolve_names(name_scope, lambda_defs, session, used_names)?;
                    name_scope.pop_names();

                    let mut lambda_def = FuncDef::create_anonymous_function(
                        args.clone(),
                        *expr.clone(),
                        self.span,
                        lambda_id,
                        session,
                    );

                    if let Some(names) = lambda_def.get_all_foreign_names() {
                        let names = names.into_iter().collect::<Vec<_>>();
                        self.kind = ExprKind::Value(
                            ValueKind::Closure(
                                lambda_def.name,
                                names.clone(),
                            )
                        );

                        lambda_def.kind = FuncKind::Closure(names);
                    }

                    else {
                        self.kind = ExprKind::Value(
                            ValueKind::Identifier(lambda_def.name, NameOrigin::Local)
                        );
                    }

                    session.add_warnings(lambda_def.get_unused_name_warnings(used_names));

                    // No hash collision between programmer-defined names and newly generated name: the new ones have special characters
                    // But there may be collisions between newly generated ones -> TODO: what then?
                    if let Some(_) = lambda_defs.insert(lambda_def.name, lambda_def) {
                        todo!();
                    }

                    Ok(())
                },
                ValueKind::Block { defs, value, id } => {
                    name_scope.push_names(defs, NameScopeKind::Block(*id));

                    for BlockDef { value, ty, .. } in defs.iter_mut() {
                        value.resolve_names(name_scope, lambda_defs, session, used_names)?;

                        if let Some(ty) = ty {
                            ty.resolve_names(name_scope, lambda_defs, session, used_names)?;
                        }
                    }

                    value.resolve_names(name_scope, lambda_defs, session, used_names)?;
                    name_scope.pop_names();

                    Ok(())
                },
            },
            ExprKind::Prefix(_, operand) | ExprKind::Postfix(_, operand) => operand.resolve_names(name_scope, lambda_defs, session, used_names),
            ExprKind::Branch(cond, b1, b2) => {
                cond.resolve_names(name_scope, lambda_defs, session, used_names)?;
                b1.resolve_names(name_scope, lambda_defs, session, used_names)?;
                b2.resolve_names(name_scope, lambda_defs, session, used_names)?;

                Ok(())
            },
            ExprKind::Call(f, args) => {
                f.resolve_names(name_scope, lambda_defs, session, used_names)?;

                for arg in args.iter_mut() {
                    arg.resolve_names(name_scope, lambda_defs, session, used_names)?;
                }

                Ok(())
            },
            ExprKind::Infix(op, o1, o2) => match op {

                // `a.b.c` -> `a` has to be resolved, but the others shall not
                InfixOp::Path => o1.resolve_names(name_scope, lambda_defs, session, used_names),
                _ => {
                    o1.resolve_names(name_scope, lambda_defs, session, used_names)?;
                    o2.resolve_names(name_scope, lambda_defs, session, used_names)
                }
            },
        }
    }

    pub fn get_all_foreign_names(
        &self,
        curr_func_id: UID,
        buffer: &mut HashSet<(InternedString, NameOrigin)>,
        curr_blocks: &mut Vec<UID>,
    ) {
        match &self.kind {
            ExprKind::Value(v) => match v {
                ValueKind::Identifier(name, origin) => match origin {
                    NameOrigin::FuncArg(id) if *id != curr_func_id => {
                        buffer.insert((*name, *origin));
                    },
                    NameOrigin::BlockDef(id) if !curr_blocks.contains(id) => {
                        buffer.insert((*name, *origin));
                    },
                    NameOrigin::NotKnownYet => {
                        // All the name has to be already resolved
                        panic!("Internal Compiler Error D0D2C11F711");
                    },
                    _ => {}
                },
                ValueKind::Integer(_)
                | ValueKind::Real(_)
                | ValueKind::String(_) 
                | ValueKind::Bytes(_) => {},
                ValueKind::List(elements)
                | ValueKind::Tuple(elements)
                | ValueKind::Format(elements) => {
                    for element in elements.iter() {
                        element.get_all_foreign_names(curr_func_id, buffer, curr_blocks);
                    }
                },

                // nested closures are possible
                ValueKind::Closure(_, captured_variables) => {
                    for (name, origin) in captured_variables.iter() {
                        match origin {
                            NameOrigin::FuncArg(id) if *id != curr_func_id => {
                                buffer.insert((*name, *origin));
                            },
                            NameOrigin::BlockDef(id) if !curr_blocks.contains(id) => {
                                buffer.insert((*name, *origin));
                            },
                            _ => {}
                        }
                    }
                }
                ValueKind::Lambda(_, _) => {
                    // Inner lambdas have to be resolved before the outer ones, if the lambdas are nested
                    panic!("Internal Compiler Error 13D43ACBD32");
                },
                ValueKind::Block {
                    defs, id, ..
                } => {
                    curr_blocks.push(*id);

                    for BlockDef { value, ty, .. } in defs.iter() {
                        value.get_all_foreign_names(curr_func_id, buffer, curr_blocks);

                        if let Some(ty) = ty {
                            ty.get_all_foreign_names(curr_func_id, buffer, curr_blocks);
                        }
                    }

                    curr_blocks.pop().expect("Internal Compiler Error 21D5A6DAABF");
                }
            },
            ExprKind::Prefix(_, operand) | ExprKind::Postfix(_, operand) => {
                operand.get_all_foreign_names(curr_func_id, buffer, curr_blocks);
            },
            ExprKind::Branch(cond, b1, b2) => {
                cond.get_all_foreign_names(curr_func_id, buffer, curr_blocks);
                b1.get_all_foreign_names(curr_func_id, buffer, curr_blocks);
                b2.get_all_foreign_names(curr_func_id, buffer, curr_blocks);
            },
            ExprKind::Call(f, args) => {
                f.get_all_foreign_names(curr_func_id, buffer, curr_blocks);

                for arg in args.iter() {
                    arg.get_all_foreign_names(curr_func_id, buffer, curr_blocks);
                }

            },
            ExprKind::Infix(_, o1, o2) => {
                o1.get_all_foreign_names(curr_func_id, buffer, curr_blocks);
                o2.get_all_foreign_names(curr_func_id, buffer, curr_blocks);
            },
        }
    }
}