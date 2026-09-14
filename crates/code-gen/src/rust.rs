use crate::Profile;
use sodigy_bytecode::{Bytecode, CodeSection, ObjectFile};

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
