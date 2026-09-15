use crate::Profile;
use sodigy_bytecode::{Bytecode, CodeSection, ObjectFile, Terminator};

pub struct RustModule {
    pub code: String,
}

pub fn lower(mut object_file: ObjectFile, profile: Profile) -> RustModule {
    let mut funcs = vec![];

    for code in object_file.code.drain(..) {
        funcs.push(lower_code(code));
    }

    todo!()
}

fn lower_code(code: CodeSection) -> String {
    let name = format!("c{}", code.label.hex(12));
    let param_count = code.params.unwrap_or(0);
    let params = match param_count {
        ..3 => "heap: &mut Heap, x0: u32, x1: u32",
        _ => "heap: &mut Heap, x0: u32, x1: u32, xs: Vec<u8>",
    };
    let mut body = vec![];

    if param_count > 2 {
        for i in 2..param_count {
            body.push(format!("let x{i} = xs.unchecked({});", i - 2));
        }
    }

    let basic_blocks_inspection = inspect_basic_blocks(&code.basic_blocks);
    let basic_block_count = match (code.basic_blocks.get(&Label::Local(0)), code.basic_blocks.len()) {
        (_, 1) => BasicBlockCount::Single,
        (Some(BasicBlock { terminator: Terminator::JumpIf(_, _), .. }), 3) => BasicBlockCount::Triple,
        _ => BasicBlockCount::Multi,
    };

    for code in code.basic_blocks.iter() {
        todo!()
    }

    let body = body.concat();
    format!(r#"
unsafe fn {name}({params}) -> CodeResult {{{body}}}
"#)
}

fn lower_bytecode(b: &Bytecode) -> String {
    todo!()
}

fn inspect_basic_blocks(basic_blocks: &HashMap<Label, BasicBlock>) -> BasicBlocksInspection {
    for (label, basic_block) in basic_blocks.iter() {
        for bytecode in basic_block.code.iter() {
            if let Some(ssa) = bytecode.
        }
    }
}
