use crate::{Bytecode, Executable, Label, Session, Value};
use sodigy_span::Span;
use std::collections::HashMap;

impl Session {
    pub fn link(&self) -> Executable {
        let mut concated_bytecodes = vec![];
        let mut label_map: HashMap<(Span, Label), usize> = HashMap::new();
        let mut func_pointer_map: HashMap<Span, usize> = HashMap::new();

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

            // `Bytecode::Label` does nothing in runtime, but we need this in order to
            // flatten the labels.
            concated_bytecodes.push(Bytecode::Label(Label::Global(def_span)));
            func_pointer_map.insert(def_span, concated_bytecodes.len());

            for (i, bytecode) in bytecodes.iter().enumerate() {
                match bytecode {
                    Bytecode::Label(label) => {
                        label_map.insert(curr_label, concated_bytecodes.len());
                        concated_bytecodes.extend(bytecodes[last_index..i].to_vec());
                        last_index = i + 1;
                        curr_label = (def_span, *label);
                    },
                    _ => {},
                }
            }

            label_map.insert(curr_label, concated_bytecodes.len());
            concated_bytecodes.extend(bytecodes[last_index..].to_vec());
        }

        let mut curr_item_span = Span::None;

        for bytecode in concated_bytecodes.iter_mut() {
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
                Bytecode::Const { value: Value::FuncPointer { def_span, program_counter }, .. } => {
                    *program_counter = Some(*func_pointer_map.get(def_span).unwrap());
                },
                _ => {},
            }
        }

        Executable {
            asserts: self.asserts.iter().map(
                |assert| (
                    assert.name.unintern_or_default(&self.intermediate_dir),
                    *label_map.get(&(assert.keyword_span, Label::Global(assert.keyword_span))).unwrap(),
                )
            ).collect(),
            bytecodes: concated_bytecodes,
        }
    }
}
