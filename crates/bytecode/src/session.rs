use crate::{
    Assert,
    Bytecode,
    DropType,
    Func,
    Label,
    Let,
};
use sodigy_error::{Error, Warning};
use sodigy_mir::{Callable, Expr, Intrinsic, Session as MirSession};
use sodigy_span::Span;
use std::collections::HashMap;

pub struct Session {
    pub intermediate_dir: String,
    pub label_counter: u32,

    pub funcs: Vec<Func>,

    // only top-level ones
    pub asserts: Vec<Assert>,
    pub lets: Vec<Let>,

    // `Span` is the name span of func param or local value (`let`).
    pub local_values: HashMap<Span, LocalValue>,

    // When you call another function, you push the args to
    // `stack[stack_offset + i]` and increment the stack pointer
    // by `stack_offset`.
    pub stack_offset: usize,

    // key: def_span of the built-in function (in sodigy std)
    pub intrinsics: HashMap<Span, Intrinsic>,
    pub lang_items: HashMap<String, Span>,

    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,
}

#[derive(Clone, Debug)]
pub struct LocalValue {
    pub stack_offset: usize,

    // we have to drop it only once!
    pub dropped: bool,

    pub drop_type: DropType,
}

impl Session {
    pub fn from_mir(mut mir_session: MirSession) -> Self {
        Session {
            intermediate_dir: mir_session.intermediate_dir.to_string(),
            label_counter: 0,
            funcs: vec![],
            asserts: vec![],
            lets: vec![],
            local_values: HashMap::new(),
            stack_offset: 0,
            intrinsics: Intrinsic::ALL_WITH_LANG_ITEM.iter().map(
                |(intrinsic, lang_item)| (*mir_session.lang_items.get(*lang_item).unwrap(), *intrinsic)
            ).collect(),
            lang_items: mir_session.lang_items.drain().collect(),
            errors: mir_session.errors.drain(..).collect(),
            warnings: mir_session.warnings.drain(..).collect(),
        }
    }

    pub fn get_lang_item_span(&self, lang_item: &str) -> Span {
        match self.lang_items.get(lang_item) {
            Some(s) => *s,
            None => panic!("TODO: lang_item `{lang_item}`"),
        }
    }

    pub fn get_local_label(&mut self) -> Label {
        self.label_counter += 1;
        Label::Local(self.label_counter - 1)
    }

    pub fn drop_block(&mut self, names: &[Span]) {
        for name in names.iter() {
            match self.local_values.get_mut(name) {
                Some(LocalValue { dropped: false, drop_type, stack_offset }) => {
                    match drop_type {
                        DropType::Scalar => {},  // no drop
                        _ => todo!(),
                    }
                },
                _ => unreachable!(),
            }
        }
    }

    pub fn drop_all_locals(&mut self, bytecodes: &mut Vec<Bytecode>) {
        for local_value in self.local_values.values_mut() {
            let LocalValue { dropped, drop_type, stack_offset } = local_value;

            if !*dropped {
                match drop_type {
                    DropType::Scalar => {},  // no drop
                    _ => todo!(),
                }
            }

            *dropped = true;
        }
    }

    pub fn collect_local_names(&mut self, expr: &Expr, offset: usize) {
        match expr {
            Expr::Ident(_) |
            Expr::Constant(_) => {},
            Expr::If(r#if) => {
                self.collect_local_names(&r#if.cond, offset);
                self.collect_local_names(&r#if.true_value, offset);
                self.collect_local_names(&r#if.false_value, offset);
            },
            Expr::Match(r#match) => {
                self.collect_local_names(&r#match.scrutinee, offset);

                for arm in r#match.arms.iter() {
                    let pattern_name_bindings = arm.pattern.bound_names();

                    for (i, (_, def_span)) in pattern_name_bindings.iter().enumerate() {
                        self.local_values.insert(
                            *def_span,
                            LocalValue {
                                stack_offset: offset + i,
                                dropped: false,

                                // TODO: drop value!!!
                                drop_type: DropType::Scalar,
                            },
                        );
                    }

                    if let Some(guard) = &arm.guard {
                        self.collect_local_names(guard, offset + pattern_name_bindings.len());
                    }

                    self.collect_local_names(&arm.value, offset + pattern_name_bindings.len());
                }
            },
            Expr::Block(block) => {
                for (i, r#let) in block.lets.iter().enumerate() {
                    self.local_values.insert(
                        r#let.name_span,
                        LocalValue {
                            stack_offset: offset + i,
                            dropped: false,

                            // TODO: drop value!!!
                            drop_type: DropType::Scalar,
                        });
                    self.collect_local_names(&r#let.value, offset + block.lets.len());
                }

                for assert in block.asserts.iter() {
                    if let Some(note) = &assert.note {
                        self.collect_local_names(note, offset + block.lets.len());
                    }

                    self.collect_local_names(&assert.value, offset + block.lets.len());
                }

                self.collect_local_names(&block.value, offset + block.lets.len());
            },
            Expr::Field { lhs, .. } => {
                self.collect_local_names(lhs, offset);
            },
            Expr::FieldUpdate { lhs, rhs, .. } => {
                self.collect_local_names(lhs, offset);
                self.collect_local_names(rhs, offset);
            },
            Expr::Call { func, args, .. } => {
                if let Callable::Dynamic(func) = func {
                    self.collect_local_names(func, offset);
                }

                for arg in args.iter() {
                    self.collect_local_names(arg, offset);
                }
            },
        }
    }

    pub fn merge(&mut self, mut s: Session) {
        self.funcs.extend(s.funcs.drain(..));
        self.asserts.extend(s.asserts.drain(..));
        self.lets.extend(s.lets.drain(..));
        // TODO: Does it have to merge `.intrinsics` and `.lang_items`?
        self.errors.extend(s.errors.drain(..));
        self.warnings.extend(s.warnings.drain(..));
    }
}
