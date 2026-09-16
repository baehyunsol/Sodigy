use crate::Profile;
use sodigy_bytecode::{
    BasicBlock,
    Bytecode,
    CodeSection,
    ExprHash,
    GlobalLabel,
    InternedValue,
    LocalLabel,
    Memory,
    ObjectFile,
    SSA,
    Terminator,
    Value,
};
use sodigy_mir::Intrinsic;
use std::collections::HashMap;

pub struct RustModule {
    pub code: String,
}

pub fn lower(mut object_file: ObjectFile, profile: Profile) -> RustModule {
    let mut funcs = vec![];

    for (hash, value) in object_file.data.drain() {
        funcs.push(lower_data(hash, value));
    }

    for (_, code) in object_file.code.drain() {
        funcs.push(lower_code(code));
    }

    todo!()
}

fn lower_data(hash: ExprHash, value: Value) -> String {
    let name = format!("c_{}", hash.hex(12));
    format!(r#"
unsafe fn {name}(heap: &mut Heap) -> u32 {{ todo!() }}
"#)
}

const RUNNER: &'static str = r#"
pub enum CallResult {
    TailCallShort { f: Fn(&mut Heap, u32, u32) -> CallResult, x0: u32, x1: u32 },
    TailCallLong { f: Fn(&mut Heap, u32, u32, Vec<u32>) -> CallResult, x0: u32, x1: u32, xs: Vec<u32> },
    Return(u32),
    Exit(u8),
}

pub fn call(heap: &mut Heap, mut c: CallResult) -> CallResult {
    loop {
        match c {
            CallResult::TailCallShort { f, x0, x1 } => { c = f(x0, x1); },
            CallResult::TailCallLong { f, x0, x1, xs } => { c = f(x0, x1, xs); },
            CallResult::Return(n) => {
                return CallResult::Return(n);
            },
            CallResult::Exit(n) => {
                return CallResult::Exit(n);
            },
        }
    }
}
"#;

enum BasicBlockCount {
    Single,
    Triple,  // purely for optimization
    Multi,
}

fn lower_code(code: CodeSection) -> String {
    let name = format!("c_{}", code.label.hex(12));
    let param_count = code.params.unwrap_or(0);
    let params = match param_count {
        ..3 => "heap: &mut Heap, x0: u32, x1: u32",
        _ => "heap: &mut Heap, x0: u32, x1: u32, xs: Vec<u32>",
    };
    let mut body = vec![];

    if param_count > 2 {
        for i in 2..param_count {
            body.push(format!("let x{i} = xs.get_unchecked({});", i - 2));
        }
    }

    let basic_blocks_inspection = inspect_basic_blocks(code.label, &code.basic_blocks);
    let basic_block_count = match (code.basic_blocks.get(&LocalLabel::start()), code.basic_blocks.len()) {
        (_, 1) => BasicBlockCount::Single,
        (Some(BasicBlock { terminator: Terminator::JumpIf { .. }, .. }), 3) => BasicBlockCount::Triple,
        _ => BasicBlockCount::Multi,
    };
    let mut body: Vec<String> = vec![];

    match (basic_block_count, basic_blocks_inspection.has_recursion) {
        (BasicBlockCount::Single, true) => {  // loop { stmts; }  # tail-call to self will continue the loop
            todo!()
        },
        (BasicBlockCount::Single, false) => {  // { stmts; }
            let basic_block = code.basic_blocks.get(&LocalLabel::start()).unwrap();
            lower_basic_block(basic_block, true, 4, &mut body);
        },
        (BasicBlockCount::Triple, true) => {  // loop { stmts; }
            todo!()
        },
        (BasicBlockCount::Triple, false) => {  // { stmts; if cond { stmts; } else { stmts; } }
            let init_block = code.basic_blocks.get(&LocalLabel::start()).unwrap();
            let Terminator::JumpIf { value: cond, t, f } = &init_block.terminator else { unreachable!() };
            lower_basic_block(init_block, false, 4, &mut body);
            body.push(format!("    if {} == 0 {{", ssa_to_rvalue(*cond)));

            let false_block = code.basic_blocks.get(f).unwrap();
            lower_basic_block(false_block, true, 8, &mut body);
            body.push(String::from("    } else {"));

            let true_block = code.basic_blocks.get(t).unwrap();
            lower_basic_block(true_block, true, 8, &mut body);
            body.push(String::from("    }"));
        },
        (BasicBlockCount::Multi, true) => {  // loop: 'recursion { let mut label = 0; loop: 'basic_blocks { match label { 0 => { stmts; }, .. } } }
            todo!()
        },
        (BasicBlockCount::Multi, false) => {  // { let mut label = 0; loop { match label { 0 => { stmts; }, .. } } }
            body.push(format!("    let mut label = {};", LocalLabel::start().index()));
            body.push(String::from("    loop {"));
            body.push(String::from("        match label {"));

            for (label, basic_block) in code.basic_blocks.iter() {
                body.push(format!("            {} => {{", label.index()));
                lower_basic_block(basic_block, true, 16, &mut body);
                body.push(String::from("            },"));
            }

            body.push(String::from("        }"));  // match
            body.push(String::from("    }"));  // loop
        },
    }

    let body = body.join("\n");
    format!(r#"
unsafe fn {name}({params}) -> CallResult {{
{body}
}}
"#)
}

struct BasicBlocksInspection {
    has_recursion: bool,
}

fn inspect_basic_blocks(global_label: GlobalLabel, basic_blocks: &HashMap<LocalLabel, BasicBlock>) -> BasicBlocksInspection {
    let mut has_recursion = false;

    for (label, basic_block) in basic_blocks.iter() {
        if let Terminator::TailCall { func, .. } = &basic_block.terminator && *func == global_label {
            has_recursion = true;
        }
    }

    BasicBlocksInspection { has_recursion }
}

fn lower_basic_block(
    basic_block: &BasicBlock,
    lower_terminator: bool,
    indent: usize,
    lines: &mut Vec<String>,
) {
    let indent_s = " ".repeat(indent);

    for code in basic_block.code.iter() {
        lower_bytecode(code, indent, lines);
    }

    if lower_terminator {
        match &basic_block.terminator {
            Terminator::Jump(n) => {
                lines.push(format!("{indent_s}label = {};", n.index()));
            },
            Terminator::TailCall { func, args } => {
                if args.len() < 3 {
                    lines.push(format!(
                        "{indent_s}return CallResult::TailCallShort {{ f: c_{}, x0: {}, x1: {} }};",
                        func.hex(12),
                        args.get(0).map(|i| ssa_to_rvalue(*i)).unwrap_or_else(|| String::from("0")),
                        args.get(1).map(|i| ssa_to_rvalue(*i)).unwrap_or_else(|| String::from("0")),
                    ));
                }

                else {
                    lines.push(format!(
                        "{indent_s}return CallResult::TailCallLong {{ f: c_{}, x0: {}, x1: {}, xs: vec![{}] }};",
                        func.hex(12),
                        ssa_to_rvalue(args[0]),
                        ssa_to_rvalue(args[1]),
                        args[2..].iter().map(|i| ssa_to_rvalue(*i)).collect::<Vec<_>>().join(", "),
                    ));
                }
            },
            Terminator::TailCallDynamic { .. } => todo!(),
            Terminator::JumpIf { value, t, f } => {
                lines.push(format!(
                    "{indent_s}if {} == 0 {{ label = {}; }} else {{ label = {}; }}",
                    ssa_to_rvalue(*value),
                    f.index(),
                    t.index(),
                ));
            },
            Terminator::TryInitGlobal { global, label } => {
                lines.push(format!(
                    "{indent_s}if !heap.global_values.contains_key(0x{}) {{ c_{}(heap); }}\n{indent_s}label = {};",
                    global.hex(12),
                    global.hex(12),
                    label.index(),
                ));
            },
            Terminator::Return(src) => {
                lines.push(format!("{indent_s}return CallResult::Return({});", ssa_to_rvalue(*src)));
            },
        }
    }
}

fn lower_bytecode(
    bytecode: &Bytecode,
    indent: usize,
    lines: &mut Vec<String>,
) {
    let indent_s = " ".repeat(indent);

    match bytecode {
        Bytecode::Const { value, dst, .. } => match value {
            InternedValue::Interned(h) => {
                lines.push(format!("{indent_s}{} = d_{}(heap);", to_lvalue(dst), h.hex(12)));
            },
            InternedValue::Scalar(n) => {
                lines.push(format!("{indent_s}{} = {n};", to_lvalue(dst)));
            },
            InternedValue::FuncPointer(_) => todo!(),
        },
        Bytecode::Move { src, dst } => {
            lines.push(format!("{indent_s}{} = {};", to_lvalue(dst), to_rvalue(src)));
        },
        Bytecode::Phi { pair: (a, b), dst } => todo!(),
        Bytecode::Call { args, dst, .. } |
        Bytecode::CallDynamic { args, dst, .. } => {
            let Some(dst) = dst else { unreachable!() };
            let f = match bytecode {
                Bytecode::Call { func, .. } => format!("c_{}", func.hex(12)),
                Bytecode::CallDynamic { func, .. } => todo!(),
                _ => unreachable!(),
            };

            if args.len() < 3 {
                lines.push(format!(
                    "{indent_s}let c = CallResult::TailCallShort {{ f: {f}, x0: {}, x1: {} }};",
                    args.get(0).map(|i| ssa_to_rvalue(*i)).unwrap_or_else(|| String::from("0")),
                    args.get(1).map(|i| ssa_to_rvalue(*i)).unwrap_or_else(|| String::from("0")),
                ));
            }

            else {
                lines.push(format!(
                    "{indent_s}let c = CallResult::TailCallLong {{ f: {f}, x0: {}, x1: {}, xs: vec![{}] }};",
                    ssa_to_rvalue(args[0]),
                    ssa_to_rvalue(args[1]),
                    args[2..].iter().map(|i| ssa_to_rvalue(*i)).collect::<Vec<_>>().join(", "),
                ));
            }

            lines.push(format!("{indent_s}match call(heap, c) {{"));
            lines.push(format!("{indent_s}    CallResult::TailCallShort {{ .. }} | CallResult::TailCallLong {{ .. }} => unreachable!(),"));
            lines.push(format!("{indent_s}    CallResult::Return(n) => {{ {} = n; }},", to_lvalue(dst)));
            lines.push(format!("{indent_s}    CallResult::Exit(n) => {{ return CallResult::Exit(n); }},"));
            lines.push(format!("{indent_s}}}"));
        },
        Bytecode::JumpIf { .. } => unreachable!(),
        Bytecode::TryInitGlobal { .. } => unreachable!(),
        Bytecode::Label(_) => unreachable!(),
        Bytecode::Return(_) => unreachable!(),
        Bytecode::Update { .. } => todo!(),
        Bytecode::Intrinsic { intrinsic, args, dst, .. } => match intrinsic {
            Intrinsic::AddScalar => {
                lines.push(format!("{indent_s}{} = {} + {};", to_lvalue(dst), ssa_to_rvalue(args[0]), ssa_to_rvalue(args[1])));
            },
            Intrinsic::Exit => {
                lines.push(format!("{indent_s}return CallResult::Exit({});", ssa_to_rvalue(args[0])));
            },
            _ => panic!("TODO: {intrinsic:?}"),
        },
        Bytecode::InitTuple { elements, dst, .. } => {
            lines.push(format!("{indent_s}{} = heap.init_tuple({elements});", to_lvalue(dst)));
        },
        Bytecode::InitList { elements, dst, .. } => {
            lines.push(format!("{indent_s}{} = heap.init_list({elements});", to_lvalue(dst)));
        },

        // These are nops and I'll remove these soon.
        Bytecode::PushDebugInfo { .. } => {},
        Bytecode::PopDebugInfo => {},
        b => panic!("TODO: {b:?}"),
    }
}

fn to_lvalue(memory: &Memory) -> String {
    match memory {
        Memory::Return => todo!(),
        Memory::SSA(i) => format!("let x{}", i.to_u32()),
        Memory::Heap { .. } => todo!(),
        _ => todo!(),
    }
}

fn to_rvalue(memory: &Memory) -> String {
    match memory {
        Memory::Return => String::from("_ret"),
        Memory::SSA(i) => format!("x{}", i.to_u32()),
        Memory::Heap { ptr, offset } => format!(
            "heap.data.get_unchecked({} as usize{})",
            ssa_to_rvalue(*ptr),
            if *offset == 0 { String::new() } else { format!(" + {offset}") },
        ),
        _ => panic!("TODO: {memory:?}"),
    }
}

fn ssa_to_rvalue(ssa: SSA) -> String {
    format!("x{}", ssa.to_u32())
}
