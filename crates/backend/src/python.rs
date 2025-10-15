use crate::{CodeGenConfig, CodeGenMode};
use sodigy_fs_api::{
    FileError,
    WriteMode,
    write_string,
};
use sodigy_lir::{
    self as lir,
    Bytecode,
    Const,
    Label,
    Register,
};
use sodigy_mir::Intrinsic;
use sodigy_number::{InternedNumber, InternedNumberValue};
use sodigy_span::Span;
use sodigy_string::unintern_string;
use std::collections::HashMap;

pub fn python_code_gen(
    output_path: &str,
    bytecode: &HashMap<u32, Vec<Bytecode>>,
    session: &lir::Session,
    config: &CodeGenConfig,
) -> Result<(), FileError> {
    let mut lines = vec![];
    let mut indent;
    let mut main_func_label = None;
    let mut help_comment_map = HashMap::new();
    let capture_output = matches!(config.mode, CodeGenMode::Test);

    if config.label_help_comment {
        for func in session.funcs.iter() {
            let func_name = unintern_string(func.name, &config.intern_str_map_dir).unwrap().unwrap();
            help_comment_map.insert(func.label_id.unwrap(), format!("fn {}", String::from_utf8_lossy(&func_name)));

            // TODO: what if there's an inline function named "main"? Do I have to restrict this? maybe...
            if func_name == b"main" {
                main_func_label = Some(func.label_id.unwrap());
            }
        }

        for r#let in session.lets.iter() {
            let let_name = unintern_string(r#let.name, &config.intern_str_map_dir).unwrap().unwrap();
            help_comment_map.insert(r#let.label_id.unwrap(), format!("let {}", String::from_utf8_lossy(&let_name)));
        }

        for assert in session.asserts.iter() {
            help_comment_map.insert(assert.label_id.unwrap(), String::from("assertion"));
        }
    }

    indent = " ".repeat(0);

    if capture_output {
        lines.push(format!("{indent}stdout=[]"));
        lines.push(format!("{indent}stderr=[]"));
    }

    lines.push(format!("{indent}def run(l):"));  // returns True if there's no error

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

    for (i, (id, bytecode)) in bytecode.iter().enumerate() {
        indent = " ".repeat(8);

        if let Some(comment) = help_comment_map.get(id) {
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
                        lines.push(format!("{indent}{}.append({})", place(dst), py_value(value, &config.intern_str_map_dir)));
                    },
                    Register::Return => {
                        lines.push(format!("{indent}ret={}", py_value(value, &config.intern_str_map_dir)));
                    },
                    Register::Const(_) => {
                        lines.push(format!("{indent}{}={}", place(dst), py_value(value, &config.intern_str_map_dir)));
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
                    Intrinsic::IntegerAdd => {
                        lines.push(format!("{indent}ret=c0[-1]+c1[-1]"));
                    },
                    Intrinsic::IntegerSub => {
                        lines.push(format!("{indent}ret=c0[-1]-c1[-1]"));
                    },
                    Intrinsic::IntegerMul => {
                        lines.push(format!("{indent}ret=c0[-1]*c1[-1]"));
                    },
                    Intrinsic::IntegerDiv => {
                        lines.push(format!("{indent}ret=c0[-1]//c1[-1]"));
                    },
                    Intrinsic::IntegerEq => {
                        lines.push(format!("{indent}ret=c0[-1]==c1[-1]"));
                    },
                    Intrinsic::IntegerGt => {
                        lines.push(format!("{indent}ret=c0[-1]>c1[-1]"));
                    },
                    Intrinsic::IntegerLt => {
                        lines.push(format!("{indent}ret=c0[-1]<c1[-1]"));
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
            }
        }
    }

    match config.mode {
        CodeGenMode::Test => {
            let mut anon_index = 0;
            lines.push(String::from("s,f=0,0"));
            lines.push(String::from("stderr_map={}"));

            for assert in session.asserts.iter() {
                let assert_name = match assert.name {
                    Some(name) => String::from_utf8_lossy(&unintern_string(name, &config.intern_str_map_dir).unwrap().unwrap()).to_string(),
                    None => {
                        anon_index += 1;
                        format!("anonymous_assertion_{anon_index}")
                    },
                };
                lines.push(format!("stdout,stderr=[],[]"));
                lines.push(format!("if run({}):", assert.label_id.unwrap()));
                lines.push(format!("    s+=1"));
                lines.push(format!("    print({assert_name:?}+': \\033[32mPass\\033[0m')"));
                lines.push(format!("else:"));
                lines.push(format!("    f+=1"));
                lines.push(format!("    print({assert_name:?}+': \\033[31mFail\\033[0m')"));
                lines.push(format!("    stderr_map[{assert_name:?}]=''.join(stderr)"));
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
            lines.push(format!("run({})", main_func_label.unwrap()));
        },
    }

    write_string(
        output_path,
        &lines.join("\n"),
        WriteMode::CreateOrTruncate,
    )
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

fn py_value(v: &Const, dictionary: &str) -> String {
    match v {
        Const::String { s, binary: true } => todo!(),
        Const::String { s, binary: false } => format!(
            "{:?}",
            String::from_utf8_lossy(&unintern_string(*s, dictionary).unwrap().unwrap()),
        ),
        Const::Number(InternedNumber { value, is_integer }) => match value {
            InternedNumberValue::SmallInteger(n) => match is_integer {
                true => format!("{n}"),
                false => format!("{n}.0"),
            },
            InternedNumberValue::SmallRatio { denom, numer } => format!("{numer}/{denom}"),
        },
    }
}

// TODO
fn hash_span(s: &Span) -> String {
    match s {
        Span::Range { file: _, start, end } => format!("_|{start}|{end}"),
        _ => todo!(),
    }
}
