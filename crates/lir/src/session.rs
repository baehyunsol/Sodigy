use crate::{
    Assert,
    Bytecode,
    DropType,
    Executable,
    Func,
    Label,
    Let,
};
use sodigy_error::{Error, Warning};
use sodigy_mir::{Callable, Expr, Intrinsic, Session as MirSession};
use sodigy_session::Session as SodigySession;
use sodigy_span::Span;
use sodigy_string::unintern_string;
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
}

#[derive(Clone, Debug)]
pub struct LocalValue {
    pub stack_offset: usize,

    // we have to drop it only once!
    pub dropped: bool,

    pub drop_type: DropType,
}

impl Session {
    pub fn from_mir(mir_session: &MirSession) -> Self {
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
            lang_items: mir_session.lang_items.clone(),
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
            Expr::Identifier(_) |
            Expr::Number { .. } |
            Expr::String { .. } |
            Expr::Char { .. } |
            Expr::Byte { .. } => {},
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
            Expr::MatchFsm(match_fsm) => todo!(),
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
            Expr::Path { lhs, .. } => {
                self.collect_local_names(lhs, offset);
            },
            Expr::FieldModifier { lhs, rhs, .. } => {
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

    pub fn into_executable(&self) -> Executable {
        let mut result = vec![];
        let mut label_map = HashMap::new();

        for (def_span, bytecodes) in self.asserts.iter().map(
            |assert| (assert.keyword_span, &assert.bytecodes)
        ).chain(
            self.lets.iter().map(
                |r#let| (r#let.name_span, &r#let.bytecodes)
            )
        ).chain(
            self.funcs.iter().map(
                |func| (func.name_span, &func.bytecodes)
            )
        ) {
            let mut curr_label = (def_span, Label::Global(def_span));
            let mut last_index = 0;

            // It does nothing in runtime, but we need this in order to flatten the labels.
            result.push(Bytecode::Label(Label::Global(def_span)));

            for (i, bytecode) in bytecodes.iter().enumerate() {
                match bytecode {
                    Bytecode::Label(label) => {
                        label_map.insert(curr_label, result.len());
                        result.extend(bytecodes[last_index..i].to_vec());
                        last_index = i + 1;
                        curr_label = (def_span, *label);
                    },
                    _ => {},
                }
            }

            label_map.insert(curr_label, result.len());
            result.extend(bytecodes[last_index..].to_vec());
        }

        let mut curr_item_span = Span::None;

        for bytecode in result.iter_mut() {
            match bytecode {
                Bytecode::Jump(label) |
                Bytecode::JumpIf { label, .. } |
                Bytecode::JumpIfUninit { label, .. } |
                Bytecode::PushCallStack(label) => {
                    let flattened_index = match *label {
                        Label::Local(ll) => label_map.get(&(curr_item_span, *label)).unwrap(),
                        Label::Global(s) => match label_map.get(&(s, Label::Global(s))) {
                            Some(i) => i,
                            None => panic!("Internal Compiler Error: Cannot find bytecode of {s:?}. Perhaps it's defined as a built-in in Sodigy, but not implemented in the compiler?"),
                        },
                        Label::Flatten(_) => unreachable!(),
                    };

                    *label = Label::Flatten(*flattened_index);
                },
                Bytecode::Label(Label::Global(def_span)) => {
                    curr_item_span = *def_span;
                },
                _ => {},
            }
        }

        Executable {
            asserts: self.asserts.iter().map(
                |assert| (
                    String::from_utf8_lossy(&unintern_string(assert.name, &self.intermediate_dir).unwrap().unwrap()).to_string(),
                    *label_map.get(&(assert.keyword_span, Label::Global(assert.keyword_span))).unwrap(),
                )
            ).collect(),
            bytecodes: result,
        }
    }
}

impl SodigySession for Session {
    fn get_errors(&self) -> &[Error] {
        &[]
    }

    fn get_warnings(&self) -> &[Warning] {
        &[]
    }

    fn get_intermediate_dir(&self) -> &str {
        &self.intermediate_dir
    }
}
