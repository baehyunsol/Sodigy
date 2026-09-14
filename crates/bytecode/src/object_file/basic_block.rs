use crate::{Bytecode, Memory, Label, SSA};
use sodigy_span::Span;
use std::collections::HashMap;

pub struct BasicBlock {
    pub label: Label,
    pub code: Vec<Bytecode>,
    pub terminator: Terminator,
    pub terminator_debug_info: Option<Box<Span>>,
}

pub enum Terminator {
    Jump(Label),
    TailCall {
        func: Label,
        args: Vec<SSA>,
    },
    TailCallDynamic {
        func: SSA,
        args: Vec<SSA>,
    },
    JumpIf(Memory, Label),
    Return(SSA),
}

pub fn to_basic_blocks(bytecodes: &mut Vec<Bytecode>) -> HashMap<Label, BasicBlock> {
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
            Bytecode::JumpIf { value, label, debug_info } => {
                basic_blocks.insert(
                    curr_label.clone().unwrap(),
                    BasicBlock {
                        label: curr_label.unwrap(),
                        code: std::mem::take(&mut curr_code),
                        terminator: Terminator::JumpIf(value, label),
                        terminator_debug_info: debug_info,
                    },
                );
                curr_label = None;
            },
            Bytecode::Label(label) => {
                assert_eq!(curr_label, None);
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

    basic_blocks
}
