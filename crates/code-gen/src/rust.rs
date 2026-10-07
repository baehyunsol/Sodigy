use crate::Profile;
use sodigy_bytecode::{
    Bytecode,
    ExprHash,
    GlobalLabel,
    InternedValue,
    LocalLabel,
    Memory,
    SSA,
    Value,
};
use sodigy_error::{Error, ErrorKind, Warning};
use sodigy_mir::Intrinsic;
use sodigy_object_file::{
    BasicBlock,
    Code,
    ObjectFile,
    Terminator,
};
use sodigy_span::SpanHash;
use std::collections::HashMap;

mod inspect;
mod session;

use inspect::{BasicBlocksInspection, Shape, inspect_basic_blocks};
use session::Session;

pub struct RustModule {
    pub code: String,
}

pub fn lower(
    mut object_file: ObjectFile,
    profile: Profile,
    errors: &mut Vec<Error>,
    warnings: &mut Vec<Warning>,
) -> RustModule {
    let mut funcs = vec![];

    let mut data: Vec<(ExprHash, Value)> = object_file.data.drain().collect();
    data.sort_by_key(|(h, _)| *h);

    for (hash, value) in data.into_iter() {
        funcs.push(lower_data(hash, value));
    }

    let mut code: Vec<(GlobalLabel, Code)> = object_file.code.drain().collect();
    code.sort_by_key(|(l, _)| *l);

    for (_, code) in code.into_iter() {
        funcs.push(lower_code(code));
    }

    funcs.push(lower_main(object_file, profile, errors, warnings));

    // dependencies
    funcs.push(RUNNER.to_string());
    funcs.push(HEAP.to_string());
    funcs.push(INT.to_string());

    RustModule {
        code: funcs.join("\n\n"),
    }
}

const RUNNER: &str = include_str!("../rust-runtime-src/run.rs");
const HEAP: &str = include_str!("../rust-runtime-src/heap.rs");
const INT: &str = include_str!("../rust-runtime-src/int.rs");

fn lower_main(object_file: ObjectFile, profile: Profile, errors: &mut Vec<Error>, warnigs: &mut Vec<Warning>) -> String {
    let mut body = vec![];

    match profile {
        Profile::Run => match object_file.main_entry {
            Some(m) => todo!(),
            None => {
                errors.push(Error {
                    kind: ErrorKind::CannotFindEntryPoint,
                    spans: vec![],
                    note: None,
                });
            },
        },
        Profile::Test => {
            body.push(String::from(r#"        let mut heap = Heap::new();
        let mut ever_failed = false;
        let samples: Vec<(&'static str, unsafe fn(&mut Heap, u32, u32) -> CallResult)> = vec!["#));

            for assert in object_file.asserts.iter() {
                body.push(format!("            ({:?}, c_{}),", assert.name, assert.label.hex(20)));
            }

            // TODO: filter assertions
            body.push(String::from(r#"        ];
        for (name, f) in samples {
            let c = CallResult::TailCallShort { f, x0: 0, x1: 0 };
            let fail = match call(&mut heap, c) {
                CallResult::TailCallShort { .. } | CallResult::TailCallLong { .. } => unreachable!(),
                CallResult::Return(_) | CallResult::Exit(0) => false,
                CallResult::Exit(_) => {
                    heap = Heap::new();
                    ever_failed = true;
                    true
                },
            };

            println!("assertion `{name}`: {}", if fail { "fail" } else { "pass" });
        }

        if ever_failed {
            std::process::ExitCode::from(22)
        } else {
            std::process::ExitCode::SUCCESS
        }"#,
            ));
        },
    }

    let body = body.join("\n");
    format!(r#"fn main() -> std::process::ExitCode {{
    unsafe {{
{body}
    }}
}}"#)
}

fn lower_data(hash: ExprHash, value: Value) -> String {
    let name = format!("d_{}", hash.hex(20));
    let mut body = vec![];

    body.push(format!("    match heap.global_values.get(&0x{}) {{", hash.hex(20)));
    body.push(format!("        Some(ptr) => *ptr,"));
    body.push(format!("        None => {{"));

    // I want to allocate everything at once.
    let mut simulated_heap = vec![];
    simulate_heap(&value, None, &mut simulated_heap, &mut 0);

    // `simulated_heap` may have multiple blocks.
    // If you can find multiple `HeapValue::Header { .. }` in `simulated_heap`, that means there are multiple blocks.
    // If there are multiple blocks, it first allocs a large enough block that can hold the blocks and split them
    // manually (e.g. inserting header and ref_count) as if the blocks are from independent allocs.
    assert!(simulated_heap.len() % 5 == 0);
    let (alloc_full, alloc_size) = alloc(simulated_heap.len() - 2);
    let mut ptr_map: HashMap<BlockId, usize> = HashMap::new();
    body.push(format!("            let ptr = {alloc_full};"));

    // If the first block's size is smaller than `simulated_heap`, that means there are multiple
    // blocks in the `simulated_heap`.
    if let Some(first_block_size) = simulated_heap[0].get_header_size() && first_block_size + 2 != simulated_heap.len() as u32 {
        body.push(format!("            *heap.data.get_unchecked_mut(ptr - 2) = 0x{:x};", 0x8000_0000 | first_block_size));

        // The pointer should point to 2 scalars after the header and there are 2 additional scalars in `simulated_heap`'s head... so the index is correct!
        for (i, value) in simulated_heap.iter().enumerate() {
            if let HeapValue::Header { id: Some(ptr), .. } = value {
                ptr_map.insert(*ptr, i);
            }
        }
    }

    body.push(format!("            *heap.data.get_unchecked_mut(ptr - 1) = 1;"));

    for (i, value) in simulated_heap.iter().skip(2).enumerate() {
        let index = format!("ptr{}", if i == 0 { String::new() } else { format!(" + {i}") });
        match value {
            HeapValue::Header { size, .. } => {
                body.push(format!("            *heap.data.get_unchecked_mut({index}) = 0x{:x};", 0x8000_0000 | size));
            },
            HeapValue::RefCount => {
                body.push(format!("            *heap.data.get_unchecked_mut({index}) = 1;"));
            },
            HeapValue::Scalar(n) => {
                body.push(format!("            *heap.data.get_unchecked_mut({index}) = 0x{n:x};"));
            },
            HeapValue::FuncPointer(_) => todo!(),
            HeapValue::Ptr(id) => {
                body.push(format!("            *heap.data.get_unchecked_mut({index}) = ptr as u32 + {};", ptr_map.get(id).unwrap()));
            },
        }
    }

    for (size, index) in calc_free_blocks(simulated_heap.len()) {
        // header
        body.push(format!("            *heap.data.get_unchecked_mut(ptr + {index}) = 0x{:x};", size - 2));

        // ref_count
        // We have to do this because there maybe garbage values from previous uses.
        body.push(format!("            *heap.data.get_unchecked_mut(ptr + {}) = 0;", index + 1));
    }

    body.push(format!("            heap.global_values.insert(0x{}, ptr as u32);", hash.hex(20)));
    body.push(format!("            ptr as u32"));
    body.push(format!("        }},"));
    body.push(format!("    }}"));

    let body = body.join("\n");
    format!(r#"unsafe fn {name}(heap: &mut Heap) -> u32 {{
{body}
}}"#,
    )
}

fn lower_code(mut code: Code) -> String {
    let inspection = inspect_basic_blocks(code.label, &code.basic_blocks);
    let mut session = Session::from_inspection(&inspection);
    let name = format!("c_{}", code.label.hex(20));
    let param_count = code.params.unwrap_or(0);
    let params = match param_count {
        0 => "heap: &mut Heap, _: u32, _: u32",
        1 => "heap: &mut Heap, x0: u32, _: u32",
        2 => "heap: &mut Heap, x0: u32, x1: u32",
        _ => "heap: &mut Heap, x0: u32, x1: u32, xs: Vec<u32>",
    };
    let mut body: Vec<String> = vec![];

    if param_count > 2 {
        for i in 2..param_count {
            body.push(format!("    let x{i}: u32 = *xs.get_unchecked({});", i - 2));
        }
    }

    let mut globals: Vec<String> = session.global_ssa.iter().filter(
        |i| i.to_u32() >= param_count as u32
    ).map(
        |i| format!("x{}", i.to_u32())
    ).chain(
        session.phi.iter().map(
            |(_, pair)| phi_register(*pair)
        )
    ).collect();

    globals.sort();
    globals.dedup();

    for global in globals.iter() {
        body.push(format!("    let mut {global}: u32 = 0;"));
    }

    match (inspection.shape, inspection.has_recursion) {
        (Shape::Single, true) => {  // loop { stmts; }  # tail-call to self will continue the loop
            body.push(String::from(r#"    todo!("Shape::Single with recursion")"#));
        },
        (Shape::Single, false) => {  // { stmts; }
            let basic_block = code.basic_blocks.get(&LocalLabel::start()).unwrap();
            lower_basic_block(basic_block, true, 4, &mut session, &mut body);
        },
        (Shape::Triple, true) => {  // loop { stmts; }
            body.push(String::from(r#"    todo!("Shape::Triple with recursion")"#));
        },
        (Shape::Triple, false) => {  // { stmts; if cond { stmts; } else { stmts; } }
            let init_block = code.basic_blocks.get(&LocalLabel::start()).unwrap();
            let Terminator::JumpIf { value: cond, t, f } = &init_block.terminator else { unreachable!() };
            lower_basic_block(init_block, false, 4, &mut session, &mut body);
            body.push(format!("    if {} == 0 {{", to_rvalue(&Memory::SSA(*cond), &session)));

            let false_block = code.basic_blocks.get(f).unwrap();
            lower_basic_block(false_block, true, 8, &mut session, &mut body);
            body.push(String::from("    } else {"));

            let true_block = code.basic_blocks.get(t).unwrap();
            lower_basic_block(true_block, true, 8, &mut session, &mut body);
            body.push(String::from("    }"));
        },
        (Shape::Multi, true) => {  // loop: 'recursion { let mut label = 0; loop: 'basic_blocks { match label { 0 => { stmts; }, .. } } }
            body.push(String::from(r#"    todo!("Shape::Multi with recursion")"#));
        },
        (Shape::Multi, false) => {  // { let mut label = 0; loop { match label { 0 => { stmts; }, .. } } }
            body.push(format!("    let mut label = {};", LocalLabel::start().index()));
            body.push(String::from("    loop {"));
            body.push(String::from("        match label {"));

            // sort these for deterministic output!
            let mut basic_blocks: Vec<(LocalLabel, BasicBlock)> = code.basic_blocks.drain().collect();
            basic_blocks.sort_by_key(|(l, _)| *l);

            for (i, (label, basic_block)) in basic_blocks.iter().enumerate() {
                if i == basic_blocks.len() - 1 {
                    body.push(String::from("            _ => {"));
                } else {
                    body.push(format!("            {} => {{", label.index()));
                }

                lower_basic_block(basic_block, true, 16, &mut session, &mut body);
                body.push(String::from("            },"));
            }

            body.push(String::from("        }"));  // match
            body.push(String::from("    }"));  // loop
        },
    }

    let body = body.join("\n");
    format!(r#"unsafe fn {name}({params}) -> CallResult {{
{body}
}}"#)
}

fn lower_basic_block(
    basic_block: &BasicBlock,
    lower_terminator: bool,
    indent: usize,
    session: &mut Session,
    lines: &mut Vec<String>,
) {
    let indent_s = " ".repeat(indent);
    let mut early_return = false;

    for code in basic_block.code.iter() {
        lower_bytecode(code, indent, session, lines, &mut early_return);
    }

    if early_return {
        return;
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
                        func.hex(20),
                        args.get(0).map(|i| to_rvalue(&Memory::SSA(*i), session)).unwrap_or_else(|| String::from("0")),
                        args.get(1).map(|i| to_rvalue(&Memory::SSA(*i), session)).unwrap_or_else(|| String::from("0")),
                    ));
                }

                else {
                    lines.push(format!(
                        "{indent_s}return CallResult::TailCallLong {{ f: c_{}, x0: {}, x1: {}, xs: vec![{}] }};",
                        func.hex(20),
                        to_rvalue(&Memory::SSA(args[0]), session),
                        to_rvalue(&Memory::SSA(args[1]), session),
                        args[2..].iter().map(|i| to_rvalue(&Memory::SSA(*i), session)).collect::<Vec<_>>().join(", "),
                    ));
                }
            },
            Terminator::TailCallDynamic { .. } => todo!(),
            Terminator::JumpIf { value, t, f } => {
                lines.push(format!(
                    "{indent_s}label = if {} == 0 {{ {} }} else {{ {} }};",
                    to_rvalue(&Memory::SSA(*value), session),
                    f.index(),
                    t.index(),
                ));
            },
            Terminator::TryInitGlobal { global, label } => {
                lines.push(format!(
                    "{indent_s}if !heap.global_values.contains_key(&0x{}) {{ c_{}(heap, 0, 0); }}\n{indent_s}label = {};",
                    global.hex(20),
                    global.hex(20),
                    label.index(),
                ));
            },
            Terminator::Return(src) => {
                lines.push(format!("{indent_s}return CallResult::Return({});", to_rvalue(&Memory::SSA(*src), session)));
            },
        }
    }
}

fn lower_bytecode(
    bytecode: &Bytecode,
    indent: usize,
    session: &mut Session,
    lines: &mut Vec<String>,
    early_return: &mut bool,
) {
    if *early_return {
        return;
    }

    let indent_s = " ".repeat(indent);

    match bytecode {
        Bytecode::Const { value, dst, .. } => match value {
            InternedValue::Interned(h) => {
                lines.push(format!("{indent_s}{} = d_{}(heap);", to_lvalue(dst, session), h.hex(20)));
            },
            InternedValue::Scalar(n) => {
                lines.push(format!("{indent_s}{} = {n};", to_lvalue(dst, session)));
            },
            InternedValue::FuncPointer(_) => {
                lines.push(format!(r#"{indent_s}{} = todo!("func-pointer");"#, to_lvalue(dst, session)));
            },
        },
        Bytecode::Move { src, dst } => {
            lines.push(format!("{indent_s}{} = {};", to_lvalue(dst, session), to_rvalue(src, session)));
        },
        Bytecode::Phi { pair, dst } => {
            lines.push(format!(
                "{indent_s}{} = {};",
                to_lvalue(dst, session),
                phi_register(*pair),
            ));
        },
        Bytecode::Jump(_) => unreachable!(),
        Bytecode::Call { args, dst, .. } |
        Bytecode::CallDynamic { args, dst, .. } => {
            let Some(dst) = dst else { unreachable!() };
            let f = match bytecode {
                Bytecode::Call { func, .. } => format!("c_{}", func.hex(20)),
                Bytecode::CallDynamic { func, .. } => format!(r#"todo!("call-dynamic-func-pointer")"#),
                _ => unreachable!(),
            };

            if args.len() < 3 {
                lines.push(format!(
                    "{indent_s}let c = CallResult::TailCallShort {{ f: {f}, x0: {}, x1: {} }};",
                    args.get(0).map(|i| to_rvalue(&Memory::SSA(*i), session)).unwrap_or_else(|| String::from("0")),
                    args.get(1).map(|i| to_rvalue(&Memory::SSA(*i), session)).unwrap_or_else(|| String::from("0")),
                ));
            }

            else {
                lines.push(format!(
                    "{indent_s}let c = CallResult::TailCallLong {{ f: {f}, x0: {}, x1: {}, xs: vec![{}] }};",
                    to_rvalue(&Memory::SSA(args[0]), session),
                    to_rvalue(&Memory::SSA(args[1]), session),
                    args[2..].iter().map(|i| to_rvalue(&Memory::SSA(*i), session)).collect::<Vec<_>>().join(", "),
                ));
            }

            lines.push(format!("{indent_s}{} = match call(heap, c) {{", to_lvalue(dst, session)));
            lines.push(format!("{indent_s}    CallResult::Return(n) => n,"));
            lines.push(format!("{indent_s}    CallResult::Exit(n) => {{ return CallResult::Exit(n); }},"));
            lines.push(format!("{indent_s}    _ => std::hint::unreachable_unchecked(),"));
            lines.push(format!("{indent_s}}};"));
        },
        Bytecode::JumpIf { .. } => unreachable!(),
        Bytecode::TryInitGlobal { .. } => unreachable!(),
        Bytecode::LoadGlobal { src, dst } => {
            lines.push(format!("{indent_s}{} = *heap.global_values.get(&0x{}).unwrap();", to_lvalue(&Memory::SSA(*dst), session), src.hex(20)));
        },
        Bytecode::StoreGlobal { src, dst } => {
            lines.push(format!("{indent_s}heap.global_values.insert(0x{}, {});", dst.hex(20), to_rvalue(&Memory::SSA(*src), session)));
        },
        Bytecode::Label(_) => unreachable!(),
        Bytecode::Return(_) => unreachable!(),
        Bytecode::Update { .. } => todo!(),
        Bytecode::Intrinsic { intrinsic, args, dst, .. } => match intrinsic {
            Intrinsic::NegInt => {
                let func = match intrinsic {
                    Intrinsic::NegInt => "neg_bi",
                    _ => unreachable!(),
                };
                lines.push(format!("{indent_s}let (is_neg, nums) = heap.inspect_int({});", to_rvalue(&Memory::SSA(args[0]), session)));
                lines.push(format!("{indent_s}let (is_neg, nums) = {func}(is_neg, nums);"));
                lines.push(format!("{indent_s}{} = heap.alloc_int(is_neg, &nums);", to_lvalue(dst, session)));
            },
            Intrinsic::AddInt |
            Intrinsic::SubInt |
            Intrinsic::MulInt |
            Intrinsic::DivInt |
            Intrinsic::RemInt => {
                let func = match intrinsic {
                    Intrinsic::AddInt => "add_bi",
                    Intrinsic::SubInt => "sub_bi",
                    Intrinsic::MulInt => "mul_bi",
                    Intrinsic::DivInt => "div_bi",
                    Intrinsic::RemInt => "rem_bi",
                    _ => unreachable!(),
                };
                lines.push(format!("{indent_s}let (lhs_is_neg, lhs_nums) = heap.inspect_int({});", to_rvalue(&Memory::SSA(args[0]), session)));
                lines.push(format!("{indent_s}let (rhs_is_neg, rhs_nums) = heap.inspect_int({});", to_rvalue(&Memory::SSA(args[1]), session)));
                lines.push(format!("{indent_s}let (is_neg, nums) = {func}(lhs_is_neg, lhs_nums, rhs_is_neg, rhs_nums);"));
                lines.push(format!("{indent_s}{} = heap.alloc_int(is_neg, &nums);", to_lvalue(dst, session)));
            },
            Intrinsic::LtInt |
            Intrinsic::EqInt |
            Intrinsic::GtInt => {
                let func = match intrinsic {
                    Intrinsic::LtInt => "lt_bi",
                    Intrinsic::EqInt => "eq_bi",
                    Intrinsic::GtInt => "gt_bi",
                    _ => unreachable!(),
                };
                lines.push(format!("{indent_s}let (lhs_is_neg, lhs_nums) = heap.inspect_int({});", to_rvalue(&Memory::SSA(args[0]), session)));
                lines.push(format!("{indent_s}let (rhs_is_neg, rhs_nums) = heap.inspect_int({});", to_rvalue(&Memory::SSA(args[1]), session)));

                // TODO: How can I guarantee that sodigy.Bool.True is always 1 and sodigy.Bool.False is always 0?
                lines.push(format!("{indent_s}{} = {func}(lhs_is_neg, lhs_nums, rhs_is_neg, rhs_nums) as u32;", to_lvalue(dst, session)));
            },
            Intrinsic::AddScalar => {
                lines.push(format!("{indent_s}{} = {} + {};", to_lvalue(dst, session), to_rvalue(&Memory::SSA(args[0]), session), to_rvalue(&Memory::SSA(args[1]), session)));
            },
            Intrinsic::SubScalar => {
                lines.push(format!("{indent_s}{} = {} - {};", to_lvalue(dst, session), to_rvalue(&Memory::SSA(args[0]), session), to_rvalue(&Memory::SSA(args[1]), session)));
            },
            Intrinsic::MulScalar => {
                lines.push(format!("{indent_s}{} = {} * {};", to_lvalue(dst, session), to_rvalue(&Memory::SSA(args[0]), session), to_rvalue(&Memory::SSA(args[1]), session)));
            },
            Intrinsic::DivScalar => {
                lines.push(format!("{indent_s}{} = {} / {};", to_lvalue(dst, session), to_rvalue(&Memory::SSA(args[0]), session), to_rvalue(&Memory::SSA(args[1]), session)));
            },
            Intrinsic::RemScalar => {
                lines.push(format!("{indent_s}{} = {} % {};", to_lvalue(dst, session), to_rvalue(&Memory::SSA(args[0]), session), to_rvalue(&Memory::SSA(args[1]), session)));
            },
            Intrinsic::LtScalar => {
                lines.push(format!("{indent_s}{} = ({} < {}) as u32;", to_lvalue(dst, session), to_rvalue(&Memory::SSA(args[0]), session), to_rvalue(&Memory::SSA(args[1]), session)));
            },
            Intrinsic::EqScalar => {
                lines.push(format!("{indent_s}{} = ({} == {}) as u32;", to_lvalue(dst, session), to_rvalue(&Memory::SSA(args[0]), session), to_rvalue(&Memory::SSA(args[1]), session)));
            },
            Intrinsic::GtScalar => {
                lines.push(format!("{indent_s}{} = ({} > {}) as u32;", to_lvalue(dst, session), to_rvalue(&Memory::SSA(args[0]), session), to_rvalue(&Memory::SSA(args[1]), session)));
            },
            Intrinsic::Exit => {
                *early_return = true;
                lines.push(format!("{indent_s}return CallResult::Exit({} as u8);", to_rvalue(&Memory::SSA(args[0]), session)));
            },
            Intrinsic::Nop0 | Intrinsic::Nop1 => {
                // Some bytecode might read the return value of this operation.
                // So the rust compiler won't let me compile this code if the result is not assigned.
                lines.push(format!("{indent_s}{} = 0;", to_lvalue(dst, session)));
            },
            i => {
                lines.push(format!(r#"{indent_s}{} = todo!("{i:?}");"#, to_lvalue(dst, session)));
            },
        },
        Bytecode::InitTuple { elements, dst, .. } => {
            lines.push(format!("{indent_s}{} = {} as u32;", to_lvalue(dst, session), alloc(*elements).0));
        },
        Bytecode::InitList { elements, dst, .. } => {
            lines.push(format!("{indent_s}let data_ptr = {};", alloc(elements + 1).0));
            lines.push(format!("{indent_s}let slice_ptr = {};", alloc(3).0));
            lines.push(format!("{indent_s}*heap.data.get_unchecked_mut(slice_ptr) = data_ptr as u32;"));
            lines.push(format!("{indent_s}*heap.data.get_unchecked_mut(slice_ptr + 1) = 0;"));
            lines.push(format!("{indent_s}*heap.data.get_unchecked_mut(slice_ptr + 2) = {elements};"));
            lines.push(format!("{indent_s}{} = slice_ptr as u32;", to_lvalue(dst, session)));

            if let Memory::SSA(dst) = dst {
                let data_ptr = session.alloc_data_ptr_index(*dst);
                lines.push(format!("{indent_s}let dp{data_ptr} = data_ptr;"));
            }
        },
        Bytecode::IncRefCount(m) => {
            lines.push(format!("{indent_s}heap.inc_rc({});", to_rvalue(m, session)));
        },
        Bytecode::DecRefCount(m) => {
            lines.push(format!("{indent_s}heap.dec_rc({});", to_rvalue(m, session)));
        },
        Bytecode::TryDrop(m, d) => {
            lines.push(format!("{indent_s}heap.try_drop({}, todo!());", to_rvalue(m, session)));
        },
        Bytecode::Breakpoint => todo!(),
    }
}

fn to_lvalue(memory: &Memory, session: &Session) -> String {
    match memory {
        Memory::SSA(i) => match session.phi.get(i) {
            Some(pair) => phi_register(*pair),
            None => if session.global_ssa.contains(i) {
                format!("x{}", i.to_u32())
            } else if session.unused_ssa.contains(i) {
                String::from("let _")
            } else {
                format!("let x{}: u32", i.to_u32())
            },
        },
        Memory::Heap { ptr, offset } => format!(
            "*heap.data.get_unchecked_mut({} as usize{})",
            to_rvalue(&Memory::SSA(*ptr), session),
            if *offset == 0 { String::new() } else { format!(" + {offset}") },
        ),
        Memory::List { ptr, offset } => match session.data_ptrs.get(ptr) {
            Some(d) => format!(
                "*heap.data.get_unchecked_mut(dp{d}{})",
                if *offset == 0 { String::new() } else { format!(" + {offset}") },
            ),
            None => unreachable!(),
        },
    }
}

fn to_rvalue(memory: &Memory, session: &Session) -> String {
    match memory {
        Memory::SSA(i) => match session.phi.get(i) {
            Some(pair) => phi_register(*pair),
            None => format!("x{}", i.to_u32()),
        },
        Memory::Heap { ptr, offset } => format!(
            "*heap.data.get_unchecked({} as usize{})",
            to_rvalue(&Memory::SSA(*ptr), session),
            if *offset == 0 { String::new() } else { format!(" + {offset}") },
        ),
        Memory::List { ptr, offset } => match session.data_ptrs.get(ptr) {
            Some(d) => format!(
                "*heap.data.get_unchecked_mut(dp{d}{})",
                if *offset == 0 { String::new() } else { format!(" + {offset}") },
            ),
            None => unreachable!(),
        },
    }
}

fn phi_register(pair: (SSA, SSA)) -> String {
    format!("p{:03x}{:03x}", pair.0.to_u32(), pair.1.to_u32())
}

fn alloc(size: usize) -> (String, usize) {
    // match size {
    //     0 => (String::from("heap.alloc_0()"), 0),
    //     ..=3 => (String::from("heap.alloc_3()"), 3),
    //     ..=8 => (String::from("heap.alloc_8()"), 8),
    //     ..=18 => (String::from("heap.alloc_18()"), 18),
    //     ..=38 => (String::from("heap.alloc_38()"), 38),
    //     ..=158 => (String::from("heap.alloc_158()"), 158),
    //     ..=638 => (String::from("heap.alloc_638()"), 638),
    //     ..=2558 => (String::from("heap.alloc_2558()"), 2558),
    //     _ => (format!("heap.alloc_large({size})"), size),
    // }
    match size {
        0 => (format!("heap.alloc(0)"), 0),
        ..=3 => (format!("heap.alloc(3)"), 3),
        ..=8 => (format!("heap.alloc(8)"), 8),
        ..=18 => (format!("heap.alloc(18)"), 18),
        ..=38 => (format!("heap.alloc(38)"), 38),
        ..=158 => (format!("heap.alloc(158)"), 158),
        ..=638 => (format!("heap.alloc(638)"), 638),
        ..=2558 => (format!("heap.alloc(2558)"), 2558),
        _ => (format!("heap.alloc({size})"), size),
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct BlockId(u32);

#[derive(Clone, Copy, Debug)]
enum HeapValue {
    Header { size: u32, id: Option<BlockId> },
    RefCount,
    Scalar(u32),
    FuncPointer(SpanHash),
    Ptr(BlockId),
}

impl HeapValue {
    pub fn get_header_size(&self) -> Option<u32> {
        match self {
            HeapValue::Header { size, .. } => Some(*size),
            _ => None,
        }
    }
}

fn simulate_heap(
    value: &Value,
    mut id: Option<BlockId>,
    buffer: &mut Vec<HeapValue>,
    block_id: &mut u32,
) {
    match value {
        Value::Scalar(_) => unreachable!(),
        Value::Int(n) => {
            let mut alloc_success = false;

            for block_size in [3, 8, 18, 38, 158, 638, 2558] {
                if n.nums.len() < block_size {
                    buffer.push(HeapValue::Header { size: block_size as u32, id });
                    buffer.push(HeapValue::RefCount);
                    buffer.push(HeapValue::Scalar(n.nums.len() as u32 | if n.is_neg { 0x8000_0000 } else { 0 }));

                    for i in 0..(block_size - 1) {
                        buffer.push(HeapValue::Scalar(*n.nums.get(i).unwrap_or(&0)));
                    }

                    alloc_success = true;
                    break;
                }
            }

            // wow... such a big integer...
            if !alloc_success {
                todo!()
            }
        },
        Value::List(values) | Value::Compound(values) => {
            let is_list = matches!(value, Value::List(_));
            let mut extra_data = vec![];
            let mut alloc_success = false;

            if is_list {
                buffer.push(HeapValue::Header { size: 3, id });
                buffer.push(HeapValue::RefCount);

                *block_id += 1;
                id = Some(BlockId(*block_id));
                buffer.push(HeapValue::Ptr(id.unwrap()));
                buffer.push(HeapValue::Scalar(0));
                buffer.push(HeapValue::Scalar(values.len() as u32));
            }

            let data_len = if is_list { values.len() + 1 } else { values.len() } as u32;

            for block_size in [3, 8, 18, 38, 158, 638, 2558] {
                if data_len <= block_size {
                    buffer.push(HeapValue::Header { size: block_size, id });
                    buffer.push(HeapValue::RefCount);

                    if is_list {
                        buffer.push(HeapValue::Scalar(values.len() as u32));
                    }

                    for value in values.iter() {
                        match value {
                            Value::Scalar(n) => {
                                buffer.push(HeapValue::Scalar(*n));
                            },
                            Value::Int(_) | Value::List(_) | Value::Compound(_) => {
                                *block_id += 1;
                                buffer.push(HeapValue::Ptr(BlockId(*block_id)));
                                simulate_heap(value, Some(BlockId(*block_id)), &mut extra_data, block_id);
                            },
                            Value::FuncPointer(ptr) => {
                                buffer.push(HeapValue::FuncPointer(*ptr));
                            },
                        }
                    }

                    for _ in 0..(block_size - data_len) {
                        buffer.push(HeapValue::Scalar(0));
                    }

                    alloc_success = true;
                    buffer.extend(extra_data);
                    break;
                }
            }

            if !alloc_success {
                todo!()
            }
        },
        Value::FuncPointer(_) => unreachable!(),
    }
}

// It allocates a single large block, split them into smaller blocks and write values to the small blocks.
// It makes the program more efficient by reducing the number of allocations, but we have to be careful not to make dangling blocks.
// For example, if it allocates a block of 40 scalars and only use the first 25 scalars, we have to make sure that the remaining
// 15 scalars are pushed to the freelists.
fn calc_free_blocks(simulated_heap_len: usize) -> Vec<(u32, usize)> {
    // Below algorithm should be identical to this match expression.
    //
    // return match simulated_heap_len {
    //     // heap.alloc will give exactly this size of block, so we don't have to create extra free blocks.
    //     5 | 10 | 20 | 40 | 160 | 640 | 2560 => vec![],
    //
    //     // It has allocated a block of 20 scalars, but using only the first 15 scalars.
    //     // We have to push the last 5 scalars to freelist_3.
    //     // The pointer returned by `heap.alloc` points to the first scalar of the data, not the header, so
    //     // we have to subtract 2, hence the index is 13, not 15.
    //     15 => vec![(5, 13)],
    //
    //     // Likewise, heap.alloc returned a block of 40 scalars, so we have to free the remaining 15 scalars.
    //     25 => vec![(10, 23), (5, 33)],
    //
    //     30 => vec![(10, 28)],
    //     35 => vec![(5, 33)],
    //     45 => vec![(20, 43), (10, 63), (5, 73)],
    //     50 => vec![(20, 48), (10, 68)],
    //     55 => vec![(20, 53), (5, 73)],
    //     // ... goes on and on
    // };

    let mut cursor = simulated_heap_len - 2;
    let target = match simulated_heap_len {
        ..=5 => 3,
        ..=10 => 8,
        ..=20 => 18,
        ..=40 => 38,
        ..=160 => 158,
        ..=640 => 638,
        ..=2560 => 2558,
        _ => todo!(),
    };
    let mut result = vec![];

    while cursor < target {
        let diff = (target - cursor) as u32;

        match diff {
            5 | 10 | 20 | 40 | 160 | 640 | 2560 => {
                result.push((diff, cursor));
                break;
            },
            15 => {
                result.push((10, cursor));
                cursor += 10;
            },
            25 | 30 | 35 => {
                result.push((20, cursor));
                cursor += 20;
            },
            ..160 => {
                result.push((40, cursor));
                cursor += 40;
            },
            ..640 => {
                result.push((160, cursor));
                cursor += 160;
            },
            ..2560 => {
                result.push((640, cursor));
                cursor += 640;
            },
            _ => {
                result.push((2560, cursor));
                cursor += 2560;
            },
        }
    }

    result
}

