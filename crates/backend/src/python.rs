use crate::{CodeGenConfig, CodeGenMode};
use sodigy_fs_api::FileError;
use sodigy_lir::{
    Bytecode,
    Const,
    ConstOrRegister,
    Executable,
    InPlaceOrRegister,
    Label,
    Offset,
    Register,
};
use sodigy_mir::Intrinsic;
use sodigy_number::{InternedNumber, InternedNumberValue};
use sodigy_span::Span;
use sodigy_string::unintern_string;

pub fn python_code_gen(
    executable: &Executable,
    config: &CodeGenConfig,
) -> Result<Vec<u8>, FileError> {
    let mut lines = vec![];
    let mut indent;
    let capture_output = matches!(config.mode, CodeGenMode::Test);

    // TODO: make it configurable
    lines.push(String::from("def deepcopy(v):"));
    lines.push(String::from("    return eval(str(v))"));

    if capture_output {
        lines.push(format!("stdout=[]"));
        lines.push(format!("stderr=[]"));
    }

    lines.push(format!("def run(l):"));  // returns True if there's no error

    indent = " ".repeat(4);

    if capture_output {
        lines.push(format!("{indent}global stdout, stderr"));
    }

    lines.push(format!("{indent}cs=[]"));
    lines.push(format!("{indent}const={}", "{}"));

    // TODO: count how many registers are used and initialize the exact number of registers
    for i in 0..10 {
        lines.push(format!("{indent}l{i}=[]"));
        lines.push(format!("{indent}c{i}=[]"));
    }

    lines.push(format!("{indent}while True:"));

    for (i, (id, bytecode)) in executable.bytecodes.iter().enumerate() {
        indent = " ".repeat(8);

        if let Some(Some(comment)) = executable.debug_info.as_ref().map(|m| m.get(id)) {
            lines.push(format!("{indent}# {comment}"));
        }

        lines.push(format!("{indent}{}if l=={id}:", if i == 0 { "" } else { "el" }));

        for b in bytecode.iter() {
            indent = " ".repeat(12);

            match b {
                Bytecode::Push { src, dst } => match dst {
                    Register::Local(_) |
                    Register::Call(_) => {
                        lines.push(format!("{indent}{}.append({})", place(dst), peek(src)));
                    },
                    Register::Return => {
                        lines.push(format!("{indent}ret={}", peek(src)));
                    },
                    Register::Const(_) => {
                        lines.push(format!("{indent}{}={}", place(dst), peek(src)));
                    },
                },
                Bytecode::PushConst { value, dst } => match dst {
                    Register::Local(_) |
                    Register::Call(_) => {
                        lines.push(format!("{indent}{}.append({})", place(dst), py_value(value, &config.intermediate_dir)));
                    },
                    Register::Return => {
                        lines.push(format!("{indent}ret={}", py_value(value, &config.intermediate_dir)));
                    },
                    Register::Const(_) => {
                        lines.push(format!("{indent}{}={}", place(dst), py_value(value, &config.intermediate_dir)));
                    },
                },
                Bytecode::Pop(src) => match src {
                    Register::Local(_) |
                    Register::Call(_) => {
                        lines.push(format!("{indent}{}.pop()", place(src)));
                    },
                    Register::Return => {
                        // there's nothing to do because Python will take care of the reference count
                    },
                    _ => unreachable!(),
                },
                Bytecode::PushCallStack(label) => match label {
                    Label::Static(n) => {
                        lines.push(format!("{indent}cs.append({n})"));
                    },
                    _ => unreachable!(),
                },
                Bytecode::PopCallStack => {
                    lines.push(format!("{indent}cs.pop()"));
                },
                Bytecode::Goto(label) => match label {
                    Label::Static(n) => {
                        lines.push(format!("{indent}l={n}"));
                        lines.push(format!("{indent}continue"));
                    },
                    _ => unreachable!(),
                },
                Bytecode::Intrinsic(intrinsic) => match intrinsic {
                    Intrinsic::AddInt => {
                        lines.push(format!("{indent}ret=c0[-1]+c1[-1]"));
                    },
                    Intrinsic::SubInt => {
                        lines.push(format!("{indent}ret=c0[-1]-c1[-1]"));
                    },
                    Intrinsic::MulInt => {
                        lines.push(format!("{indent}ret=c0[-1]*c1[-1]"));
                    },
                    Intrinsic::DivInt => {
                        lines.push(format!("{indent}ret=c0[-1]//c1[-1]"));
                    },
                    Intrinsic::RemInt => {
                        lines.push(format!("{indent}ret=c0[-1]%c1[-1]"));
                    },
                    Intrinsic::LtInt => {
                        lines.push(format!("{indent}ret=c0[-1]<c1[-1]"));
                    },
                    Intrinsic::EqInt => {
                        lines.push(format!("{indent}ret=c0[-1]==c1[-1]"));
                    },
                    Intrinsic::GtInt => {
                        lines.push(format!("{indent}ret=c0[-1]>c1[-1]"));
                    },
                    Intrinsic::Panic => {
                        lines.push(format!("{indent}return False"));
                    },
                    Intrinsic::Exit => {
                        lines.push(format!("{indent}return True"));
                    },
                    Intrinsic::Print => {
                        if capture_output {
                            lines.push(format!("{indent}stdout.append(c0[-1])"));
                        }

                        else {
                            lines.push(format!("{indent}print(c0[-1],end='')"));
                        }
                    },
                    Intrinsic::EPrint => {
                        if capture_output {
                            lines.push(format!("{indent}stderr.append(str(c0[-1]))"));
                        }

                        else {
                            lines.push(format!("{indent}import sys"));
                            lines.push(format!("{indent}print(c0[-1],file=sys.stderr,end='')"));
                        }
                    },
                },
                Bytecode::Label(_) => unreachable!(),
                Bytecode::Return => {
                    lines.push(format!("{indent}l=cs[-1]"));
                    lines.push(format!("{indent}continue"));
                },
                Bytecode::JumpIf { value: reg, label } | Bytecode::JumpIfInit { reg, label } => {
                    let Label::Static(n) = label else { unreachable!() };
                    lines.push(format!("{indent}if {}:", peek(reg)));
                    lines.push(format!("{indent}    l={n}"));
                    lines.push(format!("{indent}    continue"));
                },
                Bytecode::UpdateCompound {
                    src,
                    offset,
                    value,
                    dst,
                } => {
                    let offset = match offset {
                        Offset::Static(n) => n.to_string(),
                        Offset::Dynamic(r) => peek(r),
                    };
                    let value = match value {
                        ConstOrRegister::Const(v) => py_value(v, &config.intermediate_dir),
                        ConstOrRegister::Register(r) => peek(r),
                    };

                    match dst {
                        InPlaceOrRegister::InPlace => {
                            lines.push(format!("{indent}tmp={}", peek(src)));
                        },
                        InPlaceOrRegister::Register(_) => {
                            lines.push(format!("{indent}tmp=deepcopy({})", peek(src)));
                        },
                    }

                    lines.push(format!("{indent}tmp[{offset}]={value}"));

                    if let InPlaceOrRegister::Register(dst) = dst {
                        match dst {
                            Register::Local(_) |
                            Register::Call(_) => {
                                lines.push(format!("{indent}{}.append(tmp)", place(dst)));
                            },
                            Register::Return => {
                                lines.push(format!("{indent}ret=tmp"));
                            },
                            Register::Const(_) => {
                                lines.push(format!("{indent}{}=tmp", place(dst)));
                            },
                        }
                    }
                },
                Bytecode::ReadCompound {
                    src,
                    offset,
                    dst,
                } => {
                    let offset = match offset {
                        Offset::Static(n) => n.to_string(),
                        Offset::Dynamic(r) => peek(r),
                    };
                    lines.push(format!("{indent}tmp={}", peek(src)));
                    lines.push(format!("{indent}tmp=deepcopy(tmp[{offset}])"));

                    match dst {
                        Register::Local(_) |
                        Register::Call(_) => {
                            lines.push(format!("{indent}{}.append(tmp)", place(dst)));
                        },
                        Register::Return => {
                            lines.push(format!("{indent}ret=tmp"));
                        },
                        Register::Const(_) => {
                            lines.push(format!("{indent}{}=tmp", place(dst)));
                        },
                    }
                },
            }
        }
    }

    match config.mode {
        CodeGenMode::Test => {
            lines.push(String::from("s,f=0,0"));
            lines.push(String::from("stderr_map={}"));

            for (id, name) in executable.asserts.iter() {
                lines.push(format!("stdout,stderr=[],[]"));
                lines.push(format!("if run({}):", id));
                lines.push(format!("    s+=1"));
                lines.push(format!("    print({name:?}+': \\033[32mPass\\033[0m')"));
                lines.push(format!("else:"));
                lines.push(format!("    f+=1"));
                lines.push(format!("    print({name:?}+': \\033[31mFail\\033[0m')"));
                lines.push(format!("    stderr_map[{name:?}]=''.join(stderr)"));
            }

            lines.push(String::from("print()"));
            lines.push(String::from("if f>0:"));
            lines.push(String::from("    for name,stderr in stderr_map.items():"));
            lines.push(String::from("        print(f'---- {name} stderr ----')"));
            lines.push(String::from("        print(stderr)"));
            lines.push(String::from("    print()"));
            lines.push(String::from("print(f'passed: {s}, failed: {f}')"));
            lines.push(String::from("if f>0:"));
            lines.push(String::from("    import sys"));
            lines.push(String::from("    sys.exit(1)"));
        },
        CodeGenMode::Binary => {
            // TODO: how do we find the main function?
            // lines.push(format!("run({})", main_func_label.unwrap()));
        },
    }

    Ok(lines.join("\n").into_bytes())
}

fn place(r: &Register) -> String {
    match r {
        Register::Local(n @ 0..=9) => format!("l{n}"),
        Register::Local(n) => format!("lrs[{}]", *n - 10),
        Register::Call(n @ 0..=9) => format!("c{n}"),
        Register::Call(n) => format!("crs[{}]", *n - 10),
        Register::Return => String::from("ret"),
        Register::Const(span) => format!("const[{:?}]", hash_span(span)),
    }
}

fn peek(r: &Register) -> String {
    match r {
        Register::Local(n @ 0..=9) => format!("l{n}[-1]"),
        Register::Local(n) => format!("lrs[{}][-1]", *n - 10),
        Register::Call(n @ 0..=9) => format!("c{n}[-1]"),
        Register::Call(n) => format!("crs[{}][-1]", *n - 10),
        Register::Return => String::from("ret"),
        Register::Const(span) => format!("const.get({:?},None)", hash_span(span)),
    }
}

fn py_value(v: &Const, intermediate_dir: &str) -> String {
    match v {
        Const::Scalar(n) => format!("{n}"),
        Const::String { s, binary } => {
            let s = unintern_string(*s, intermediate_dir).unwrap().unwrap();

            if *binary {
                format!(
                    "[{}{}]",
                    s.len(),
                    s.iter().map(
                        |b| format!(",{b}")
                    ).collect::<Vec<_>>().concat(),
                )
            }

            else {
                // TODO: does the compiler guarantee that it's a valid utf-8?
                let s = String::from_utf8_lossy(&s).to_string();

                // FIXME: it's counting `.chars()` twice, but I'm too lazy
                //        to write an optimized code
                format!(
                    "[{}{}]",
                    s.chars().count(),
                    s.chars().map(
                        |c| format!(",{}", c as u32)
                    ).collect::<Vec<_>>().concat(),
                )
            }
        },
        Const::Number(InternedNumber { value, is_integer }) => match value {
            InternedNumberValue::SmallInt(n) => match is_integer {
                true => format!("{n}"),
                false => format!("{n}.0"),
            },
            InternedNumberValue::SmallRatio { denom, numer } => format!("{numer}/{denom}"),
            _ => panic!("TODO: {value:?}"),
        },
        Const::Compound(n) => format!("[None for _ in range({n})]"),
    }
}

// TODO
fn hash_span(s: &Span) -> String {
    match s {
        Span::Range { file: _, start, end } => format!("_|{start}|{end}"),
        _ => todo!(),
    }
}
