use crate::{
    Bytecode,
    DropType,
    ExprHash,
    Label,
    Memory,
    ObjectFile,
    SSA,
    Value,
};
use sodigy_error::{Error, Warning};
use sodigy_mir::{
    GlobalContext,
    Intrinsic,
    Session as MirSession,
};
use sodigy_number::InternedNumber;
use sodigy_session::SodigySession;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

pub struct Session<'hir, 'mir> {
    pub intermediate_dir: String,
    pub label_counter: u32,
    pub ssa_counter: u32,
    pub ssa_map: HashMap<Span, SSA>,

    // for data section in the object file
    pub number_to_expr_hash: HashMap<InternedNumber, ExprHash>,
    pub string_to_expr_hash: HashMap<(InternedString, /* binary: */ bool), ExprHash>,
    pub data_section: HashMap<ExprHash, Value>,

    // key: def_span of the built-in function (in sodigy std)
    pub intrinsics: HashMap<Span, Intrinsic>,

    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,
    pub object_file: ObjectFile,
    pub global_context: GlobalContext<'hir, 'mir>,
    pub debug_info: bool,
}

#[derive(Clone, Debug)]
pub struct LocalValue {
    pub stack_offset: usize,

    // we have to drop it only once!
    pub dropped: bool,

    pub drop_type: DropType,
}

impl Session<'_, '_> {
    pub fn from_mir<'hir, 'mir>(mut mir_session: MirSession<'hir, 'mir>) -> Session<'hir, 'mir> {
        Session {
            intermediate_dir: mir_session.intermediate_dir.to_string(),
            label_counter: 0,
            ssa_counter: 0,
            ssa_map: HashMap::new(),
            number_to_expr_hash: HashMap::new(),
            string_to_expr_hash: HashMap::new(),
            data_section: HashMap::new(),
            intrinsics: Intrinsic::ALL_WITH_LANG_ITEM.iter().map(
                |(intrinsic, lang_item)| (mir_session.get_lang_item_span(lang_item), *intrinsic)
            ).collect(),
            errors: mir_session.errors.drain(..).collect(),
            warnings: mir_session.warnings.drain(..).collect(),
            object_file: ObjectFile::empty(),
            global_context: mir_session.global_context,
            debug_info: true,  // TODO: make it configurable
        }
    }

    pub fn get_lang_item_span(&self, lang_item: &str) -> Span {
        match self.global_context.lang_items.unwrap().get(lang_item) {
            Some(s) => s.clone(),
            None => panic!("TODO: lang_item `{lang_item}`"),
        }
    }

    pub fn get_local_label(&mut self) -> Label {
        self.label_counter += 1;
        Label::Local(self.label_counter - 1)
    }

    pub fn get_ssa(&mut self) -> SSA {
        self.ssa_counter += 1;
        SSA(self.ssa_counter - 1)
    }

    // If `src` is already an ssa register, it returns the ssa index of `src`.
    // Otherwise, it moves `src` to an ssa register and returns the ssa index of the new register.
    pub fn move_to_ssa(&mut self, src: &Memory, bytecodes: &mut Vec<Bytecode>) -> SSA {
        match src {
            Memory::SSA(n) => *n,
            _ => {
                let ssa_reg = self.get_ssa();
                bytecodes.push(Bytecode::Move {
                    src: src.clone(),
                    dst: Memory::SSA(ssa_reg),
                });
                ssa_reg
            },
        }
    }
}
