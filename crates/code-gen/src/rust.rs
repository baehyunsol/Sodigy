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
use sodigy_error::{Error, ErrorKind, Warning};
use sodigy_mir::Intrinsic;

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

    let mut code: Vec<(GlobalLabel, CodeSection)> = object_file.code.drain().collect();
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

const RUNNER: &'static str = include_str!("../rust-runtime-src/run.rs");
const HEAP: &'static str = include_str!("../rust-runtime-src/heap.rs");
const INT: &'static str = include_str!("../rust-runtime-src/int.rs");

fn lower_main(object_file: ObjectFile, profile: Profile, errors: &mut Vec<Error>, warnigs: &mut Vec<Warning>) -> String {
    let mut body = vec![];

    match profile {
        Profile::Run => match object_file.main_entry {
            Some(m) => todo!(),
            None => {
                errors.push(Error {
                    kind: ErrorKind::CannotFindMainEntry,
                    spans: vec![],
                    note: None,
                });
            },
        },
        Profile::Test => {
            body.push(String::from(r#"        let mut heap = Heap::new();
        let mut ever_failed = false;
        let samples: Vec<(&'static str, unsafe fn(&mut Heap, u32, u32) -> CallResult)> = vec!["#));

            for (name, label) in object_file.asserts.iter() {
                body.push(format!("            ({name:?}, c_{}),", label.hex(20)));
            }

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

    match value {
        Value::Scalar(_) => unreachable!(),
        Value::Int(n) => {
            let p_len = n.nums.len() + 1;
            let mut metadata = n.nums.len();

            if n.is_neg {
                metadata |= 0x8000_0000;
            }

            body.push(format!("    let ptr: usize = heap.alloc({p_len});"));
            body.push(format!("    *heap.data.get_unchecked_mut(ptr) = 0x{metadata:x};"));

            for (i, n) in n.nums.iter().enumerate() {
                body.push(format!("    *heap.data.get_unchecked_mut(ptr + {}) = 0x{n:x};", i + 1));
            }

            body.push(String::from("    ptr as u32"));
        },
        Value::List(vs) => {
            if vs.is_empty() {
                body.push(String::from("    let data_ptr: usize = 0;"));
            } else {
                body.push(format!("    let data_ptr: usize = heap.alloc({});", vs.len() + 1));
                body.push(format!("    *heap.data.get_unchecked_mut(data_ptr) = 0x{:x};", vs.len()));

                for (i, v) in vs.iter().enumerate() {
                    match v {
                        Value::Scalar(n) => {
                            body.push(format!("    *heap.data.get_unchecked_mut(data_ptr + {}) = 0x{n:x};", i + 1));
                        },
                        _ => todo!(),
                    }
                }
            }

            body.push(format!("    let slice_ptr: usize = heap.alloc(3);"));
            body.push(format!("    *heap.data.get_unchecked_mut(slice_ptr) = data_ptr as u32;"));
            body.push(format!("    *heap.data.get_unchecked_mut(slice_ptr + 1) = 0;"));
            body.push(format!("    *heap.data.get_unchecked_mut(slice_ptr + 2) = {};", vs.len()));
            body.push(format!("    slice_ptr as u32"));
        },
        _ => {
            body.push(format!("    // {value:?}"));
            body.push(format!("    todo!()"));
        },
    }

    let body = body.join("\n");
    format!(r#"unsafe fn {name}(heap: &mut Heap) -> u32 {{
{body}
}}"#,
    )
}

fn lower_code(mut code: CodeSection) -> String {
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
    session: &Session,
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
            lines.push(format!("{indent_s}{} = heap.init_tuple({elements});", to_lvalue(dst, session)));
        },
        Bytecode::InitList { elements, dst, .. } => {
            lines.push(format!("{indent_s}{} = heap.init_list({elements});", to_lvalue(dst, session)));
        },
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
        Memory::List { ptr, offset } => format!("*heap.mut_list({}, {offset})", to_rvalue(&Memory::SSA(*ptr), session)),
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
        Memory::List { ptr, offset } => format!("heap.read_list({}, {offset})", to_rvalue(&Memory::SSA(*ptr), session)),
    }
}

fn phi_register(pair: (SSA, SSA)) -> String {
    format!("p{:03x}{:03x}", pair.0.to_u32(), pair.1.to_u32())
}
