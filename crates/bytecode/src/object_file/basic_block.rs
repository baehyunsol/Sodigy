use crate::{Bytecode, GlobalLabel, LocalLabel, Memory, SSA};
use sodigy_span::Span;
use std::collections::HashMap;

pub struct BasicBlock {
    pub label: LocalLabel,
    pub code: Vec<Bytecode>,
    pub terminator: Terminator,
    pub terminator_debug_info: Option<Box<Span>>,
}

pub enum Terminator {
    Jump(LocalLabel),

    // TODO: do we need FuncEffect? I don't know...
    TailCall {
        func: GlobalLabel,
        args: Vec<SSA>,
    },
    TailCallDynamic {
        func: SSA,
        args: Vec<SSA>,
    },

    JumpIf {
        value: Memory,
        t: LocalLabel,
        f: LocalLabel,
    },
    TryInitGlobal {
        global: GlobalLabel,
        label1: LocalLabel,
        label2: LocalLabel,
    },
    Return(SSA),
}

pub fn to_basic_blocks(bytecodes: &mut Vec<Bytecode>) -> HashMap<LocalLabel, BasicBlock> {
    let mut basic_blocks = HashMap::new();
    let mut curr_label = None;
    let mut curr_code = vec![];

    for bytecode in bytecodes.drain(..) {
        match bytecode {
            Bytecode::Jump(label) => {
                basic_blocks.insert(
                    curr_label.clone().unwrap(),
                    BasicBlock {
                        label: curr_label.unwrap(),
                        code: std::mem::take(&mut curr_code),
                        terminator: Terminator::Jump(label),
                        terminator_debug_info: None,
                    },
                );
                curr_label = None;
            },
            Bytecode::Call { func, args, dst: None, debug_info, .. } => {
                basic_blocks.insert(
                    curr_label.clone().unwrap(),
                    BasicBlock {
                        label: curr_label.unwrap(),
                        code: std::mem::take(&mut curr_code),
                        terminator: Terminator::TailCall { func, args },
                        terminator_debug_info: debug_info,
                    },
                );
                curr_label = None;
            },
            Bytecode::CallDynamic { func, args, dst: None, debug_info, .. } => {
                basic_blocks.insert(
                    curr_label.clone().unwrap(),
                    BasicBlock {
                        label: curr_label.unwrap(),
                        code: std::mem::take(&mut curr_code),
                        terminator: Terminator::TailCallDynamic { func, args },
                        terminator_debug_info: debug_info,
                    },
                );
                curr_label = None;
            },
            Bytecode::JumpIf { value, t, f, debug_info } => {
                basic_blocks.insert(
                    curr_label.clone().unwrap(),
                    BasicBlock {
                        label: curr_label.unwrap(),
                        code: std::mem::take(&mut curr_code),
                        terminator: Terminator::JumpIf { value, t, f },
                        terminator_debug_info: debug_info,
                    },
                );
                curr_label = None;
            },
            Bytecode::TryInitGlobal { def_span: _, global, label1, label2 } => {
                basic_blocks.insert(
                    curr_label.clone().unwrap(),
                    BasicBlock {
                        label: curr_label.unwrap(),
                        code: std::mem::take(&mut curr_code),
                        terminator: Terminator::TryInitGlobal { global, label1, label2 },
                        terminator_debug_info: None,
                    },
                );
                curr_label = None;
            },
            Bytecode::Label(label) => {
                if curr_label.is_some() {
                    basic_blocks.insert(
                        curr_label.clone().unwrap(),
                        BasicBlock {
                            label: curr_label.unwrap(),
                            code: std::mem::take(&mut curr_code),
                            terminator: Terminator::Jump(label.clone()),
                            terminator_debug_info: None,
                        },
                    );
                }

                curr_label = Some(label);
            },
            Bytecode::Return(src) => {
                basic_blocks.insert(
                    curr_label.clone().unwrap(),
                    BasicBlock {
                        label: curr_label.unwrap(),
                        code: std::mem::take(&mut curr_code),
                        terminator: Terminator::Return(src),
                        terminator_debug_info: None,
                    },
                );
                curr_label = None;
            },
            code => {
                curr_code.push(code);
            },
        }
    }

    assert!(curr_code.is_empty());
    basic_blocks
}
