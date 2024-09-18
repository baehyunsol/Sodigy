use crate::func::{LocalValue, MaybeInit};
use crate::expr::lower_expr;
use crate::ty::lower_ty;
use sodigy_high_ir::{self as hir, NameBindingType};
use sodigy_intern::InternedString;
use sodigy_uid::Uid;
use std::collections::HashMap;

pub struct MirSession {
    curr_lowering_func: Option<Uid>,
    local_value_table: HashMap<LocalValueSearchKey, usize>,
}

impl MirSession {
    pub fn start_lowering_func(&mut self, func: Uid) {
        assert!(self.curr_lowering_func.is_none());
        self.curr_lowering_func = Some(func);
    }

    pub fn end_lowering_func(&mut self) {
        assert!(self.curr_lowering_func.is_some());
        self.curr_lowering_func = None;
    }

    pub fn register_local_values(&mut self, func: &hir::Func) -> Result<Vec<LocalValue>, ()> {
        let mut local_values = vec![];
        let mut local_value_table = HashMap::new();

        if let Some(args) = &func.args {
            for hir::Arg { name, ty, attributes, .. } in args.iter() {
                // TODO: collect values in attributes

                let ty = match ty {
                    Some(ty) => {
                        collect_local_values(
                            ty.as_expr(),
                            &mut local_value_table,
                            &mut local_values,
                        );
                        MaybeInit::Uninit(ty.clone())
                    },
                    None => MaybeInit::None,
                };

                local_value_table.insert(
                    LocalValueSearchKey::FuncArg(name.id()),
                    local_values.len(),
                );

                local_values.push(LocalValue {
                    name: *name,
                    name_binding_type: NameBindingType::FuncArg,
                    value: MaybeInit::None,
                    ty,
                    parent_func: func.uid,
                    parent_scope: None,
                    index: local_values.len(),
                });
            }
        }

        for generic in func.generics.iter() {
            local_value_table.insert(
                LocalValueSearchKey::FuncGeneric(generic.id()),
                local_values.len(),
            );

            local_values.push(LocalValue {
                name: *generic,
                name_binding_type: NameBindingType::FuncGeneric,
                value: MaybeInit::None,
                ty: todo!(),  // Prelude::Type
                parent_func: func.uid,
                parent_scope: None,
                index: local_values.len(),
            });
        }

        collect_local_values(
            &func.return_value,
            &mut local_value_table,
            &mut local_values,
        );

        if let Some(ty) = &func.return_type {
            collect_local_values(
                ty.as_expr(),
                &mut local_value_table,
                &mut local_values,
            );
        }

        // TODO: collect_local_values in attributes

        self.local_value_table = local_value_table;
        let mut has_error = false;

        for local_value in local_values.iter_mut() {
            match &local_value.value {
                MaybeInit::Uninit(v) => {
                    // TODO: it's lowering the type twice
                    //       duplicate error messages are handled by the compiler, but it's still inefficient
                    let v = lower_expr(
                        v,
                        local_value.ty.try_unwrap_uninit(),
                        false,
                        self,
                    );

                    if let Ok(v) = v {
                        local_value.value = MaybeInit::Init(v);
                    } else {
                        has_error = true;
                    }
                },
                MaybeInit::None => {},
                MaybeInit::Init(_) => unreachable!(),
            }

            match &local_value.ty {
                MaybeInit::Uninit(ty) => {
                    let ty = lower_ty();

                    if let Ok(ty) = ty {
                        local_value.ty = MaybeInit::Init(ty);
                    } else {
                        has_error = true;
                    }
                },
                MaybeInit::None => {},
                MaybeInit::Init(_) => unreachable!(),
            }
        }

        if has_error {
            return Err(());
        }

        Ok(local_values)
    }

    pub fn get_local_value_index(&self, key: LocalValueSearchKey) -> usize {
        // It's an internal compiler error if it fails
        *self.local_value_table.get(&key).unwrap()
    }

    pub fn curr_func_uid(&self) -> Uid {
        self.curr_lowering_func.unwrap()
    }
}

#[derive(Eq, Hash, PartialEq)]
pub enum LocalValueSearchKey {
    FuncArg(InternedString),
    FuncGeneric(InternedString),
    LocalValue(Uid, InternedString),
}

fn collect_local_values(
    e: &hir::Expr,
    local_value_table: &mut HashMap<LocalValueSearchKey, usize>,
    local_values: &mut Vec<LocalValue>,
) {
    todo!()
}
