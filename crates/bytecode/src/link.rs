use crate::{Bytecode, CodeKind, Executable, Label, ObjectFile, Value};
use sodigy_span::SpanHash;
use std::collections::hash_map::{Entry, HashMap};

pub fn link(object_files: Vec<ObjectFile>) -> ObjectFile {
    let mut data = HashMap::new();
    let mut code = vec![];
    let mut main_entry = None;
    let mut asserts = vec![];

    for mut object_file in object_files.into_iter() {
        for (key, value) in object_file.data.drain(..) {
            if let Entry::Vacant(e) = data.entry(key) {
                e.insert(value);
            }
        }

        // TODO: What if there are multiple object files that have a main entry?
        if let Some(main) = object_file.main_entry {
            main_entry = Some(main);
        }

        code.extend(object_file.code.drain(..));
        asserts.extend(object_file.asserts.drain(..));
    }

    ObjectFile {
        data: data.into_iter().collect(),
        code,
        main_entry,
        asserts,
    }
}

pub fn flatten(object_file: &mut ObjectFile) -> Executable {
    let mut concated_bytecodes = vec![];
    let mut label_map: HashMap<(SpanHash, Label), usize> = HashMap::new();
    let mut func_pointer_map: HashMap<SpanHash, usize> = HashMap::new();
    let mut asserts: Vec<(String, SpanHash)> = vec![];

    for code in object_file.code.iter() {
        if let CodeKind::Assert = code.kind {
            asserts.push((code.name.clone(), code.label));
        }

        let mut curr_label = (code.label, Label::Global(code.label));
        let mut last_index = 0;

        // `Bytecode::Label` does nothing in runtime, but we need this in order to
        // flatten the labels.
        concated_bytecodes.push(Bytecode::Label(Label::Global(code.label)));
        func_pointer_map.insert(code.label, concated_bytecodes.len());

        for (i, bytecode) in code.code.iter().enumerate() {
            if let Bytecode::Label(label) = bytecode {
                label_map.insert(curr_label, concated_bytecodes.len());
                concated_bytecodes.extend(code.code[last_index..i].to_vec());
                last_index = i + 1;
                curr_label = (code.label, label.clone());
            }
        }

        label_map.insert(curr_label, concated_bytecodes.len());
        concated_bytecodes.extend(code.code[last_index..].to_vec());
    }

    let mut curr_item_span: Option<SpanHash> = None;

    for bytecode in concated_bytecodes.iter_mut() {
        match bytecode {
            Bytecode::Jump(label) |
            Bytecode::Call { func: label, .. } |
            Bytecode::JumpIf { label, .. } => {
                let flattened_index = match label {
                    Label::Local(_) => label_map.get(&(curr_item_span.unwrap(), label.clone())).unwrap(),
                    Label::Global(s) => match label_map.get(&(*s, Label::Global(*s))) {
                        Some(i) => i,
                        None => panic!("Internal Compiler Error: Cannot find bytecode of {s:?} ({}). Perhaps it's defined as a built-in in Sodigy, but not implemented in the compiler?", Label::Global(s.clone())),
                    },
                    Label::Flatten(_) => unreachable!(),
                };

                *label = Label::Flatten(*flattened_index);
            },
            Bytecode::InitOrJump { func, label, .. } => {
                for label in [func, label] {
                    let flattened_index = match label {
                        Label::Local(_) => label_map.get(&(curr_item_span.unwrap(), label.clone())).unwrap(),
                        Label::Global(s) => match label_map.get(&(*s, Label::Global(*s))) {
                            Some(i) => i,
                            None => panic!("Internal Compiler Error: Cannot find bytecode of {s:?} ({}). Perhaps it's defined as a built-in in Sodigy, but not implemented in the compiler?", Label::Global(s.clone())),
                        },
                        Label::Flatten(_) => unreachable!(),
                    };

                    *label = Label::Flatten(*flattened_index);
                }
            },
            Bytecode::Label(Label::Global(s)) => {
                curr_item_span = Some(*s);
            },
            _ => {},
        }
    }

    for (_, data) in object_file.data.iter_mut() {
        if let Value::FuncPointer { def_span, program_counter } = data {
            *program_counter = Some(*func_pointer_map.get(def_span).unwrap());
        }
    }

    let asserts = asserts.into_iter().map(
        |(name, label)| (name, *label_map.get(&(label, Label::Global(label))).unwrap())
    ).collect();

    Executable {
        data: object_file.data.drain(..).collect(),
        code: concated_bytecodes,
        main_entry: object_file.main_entry.map(|e| *label_map.get(&(e, Label::Global(e))).unwrap()),
        asserts,
    }
}
