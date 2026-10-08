use crate::{
    Assert,
    BasicBlock,
    Code,
    CodeKind,
    Entry,
    ObjectFile,
    Profile,
    Terminator,
    to_basic_blocks,
};
use sodigy_bytecode::{
    Bytecode,
    GlobalLabel,
    LocalLabel,
    Memory,
    Session as BytecodeSession,
    SSA,
};
use sodigy_error::{Error, FuncEffect, Warning};
use sodigy_mir::Intrinsic;
use sodigy_string::unintern_string;
use sodigy_utils::camel_to_snake;
use std::collections::HashMap;

pub struct Session {
    pub object_file: ObjectFile,
    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,
}

impl Session {
    pub fn from_bytecode_session(
        mut bytecode_session: BytecodeSession<'_, '_>,
        profile: Profile,
        lower_built_ins: bool,
    ) -> Session {
        let mut code = HashMap::with_capacity(bytecode_session.lets.len() + bytecode_session.funcs.len() + bytecode_session.asserts.len());
        let mut asserts = vec![];

        for mut func in bytecode_session.funcs.drain(..) {
            let label = GlobalLabel::new(func.name_span.hash());
            code.insert(
                label,
                Code {
                    label,
                    span: Some(func.name_span.clone()),
                    kind: CodeKind::Func,
                    name: String::from_utf8_lossy(&unintern_string(func.name, &bytecode_session.intermediate_dir).unwrap().unwrap()).to_string(),
                    params: Some(func.params),
                    effect: func.effect.clone(),
                    basic_blocks: to_basic_blocks(&mut func.bytecodes),
                },
            );
        }

        for mut r#let in bytecode_session.lets.drain(..) {
            let label = GlobalLabel::new(r#let.name_span.hash());
            code.insert(
                label,
                Code {
                    label,
                    span: Some(r#let.name_span.clone()),
                    kind: CodeKind::Let,
                    name: String::from_utf8_lossy(&unintern_string(r#let.name, &bytecode_session.intermediate_dir).unwrap().unwrap()).to_string(),
                    params: None,
                    effect: FuncEffect::Fn,
                    basic_blocks: to_basic_blocks(&mut r#let.bytecodes),
                },
            );
        }

        for mut assert in bytecode_session.asserts.drain(..) {
            let name = String::from_utf8_lossy(&unintern_string(assert.name, &bytecode_session.intermediate_dir).unwrap().unwrap()).to_string();
            let label = GlobalLabel::new(assert.keyword_span.hash());
            let is_std = assert.keyword_span.is_std();
            let assert_label = Assert { name: name.clone(), label, is_std };

            if !profile.has_to_check_this_assertion(&assert_label) {
                continue;
            }

            asserts.push(assert_label);
            code.insert(
                label,
                Code {
                    label,
                    span: Some(assert.keyword_span.clone()),
                    kind: CodeKind::Assert,
                    name,
                    params: None,
                    effect: FuncEffect::Fn,
                    basic_blocks: to_basic_blocks(&mut assert.bytecodes),
                },
            );
        }

        // We need code sections for built-ins when we want to create function pointers
        // for built-in functions.
        if lower_built_ins {
            for (intrinsic, lang_item) in Intrinsic::ALL_WITH_LANG_ITEM.iter() {
                let def_span = bytecode_session.global_context.get_lang_item_span(lang_item);
                let label = GlobalLabel::new(def_span.hash());
                code.insert(
                    label,
                    Code {
                        label,
                        span: Some(def_span.clone()),
                        kind: CodeKind::Func,
                        name: camel_to_snake(&format!("{intrinsic:?}")),
                        params: Some(intrinsic.num_params()),
                        effect: intrinsic.effect(),
                        basic_blocks: [(
                            LocalLabel::start(),
                            BasicBlock {
                                label: LocalLabel::start(),
                                code: vec![
                                    Bytecode::Intrinsic {
                                        intrinsic: *intrinsic,
                                        args: (0..intrinsic.num_params()).map(
                                            |i| SSA::from_u32(i as u32)
                                        ).collect(),
                                        dst: Memory::SSA(SSA::from_u32(intrinsic.num_params() as u32 + 1)),
                                        debug_info: None,
                                    },
                                ],
                                terminator: Terminator::Return(SSA::from_u32(intrinsic.num_params() as u32 + 1)),
                                terminator_debug_info: None,
                            },
                        )].into_iter().collect(),
                    },
                );
            }
        }

        let entry = match (profile, &bytecode_session.global_context.entry_point) {
            // FIXME: `global_context.entry_point` is of the entire project, but this object-file is
            //        only for a single session.
            (Profile::Run, Some(Some(e))) => Entry::Main(GlobalLabel::new(e.hash())),
            (Profile::Run, _) => Entry::NoEntry,
            (Profile::Test { .. }, _) => Entry::Asserts(asserts),
        };
        let object_file = ObjectFile {
            data: std::mem::take(&mut bytecode_session.data_section),
            code,
            entry,
        };

        Session {
            object_file,
            errors: std::mem::take(&mut bytecode_session.errors),
            warnings: std::mem::take(&mut bytecode_session.warnings),
        }
    }
}

