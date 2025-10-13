use crate::CodeGenConfig;
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
use std::collections::HashMap;

pub fn python_code_gen(
    output_path: &str,
    bytecode: HashMap<u32, Vec<Bytecode>>,
    session: &lir::Session,
    config: &CodeGenConfig,
) -> Result<(), FileError> {
    let mut lines = vec![];
    let mut indent;
    lines.push(String::from("while True:"));

    for (i, (id, bytecode)) in bytecode.iter().enumerate() {
        indent = "    ";
        lines.push(format!("{indent}{}if s == {id}:", if i == 0 { "" } else { "el" }));

        for b in bytecode.iter() {
            indent = "        ";

            match b {
                // c0.append(l0[-1]);
                // c0.append(ret);
                // ret = l0[-1]
                Bytecode::Push { src, dst } => match dst {
                    Register::Local(_) |
                    Register::Call(_) => {
                        lines.push(format!("{indent}{}.append({})", place(dst), peek(src)));
                    },
                    Register::Return => {
                        lines.push(format!("{indent}ret={}", peek(src)));
                    },
                    Register::Const(span) => todo!(),
                },
                Bytecode::PushConst { value, dst } => match dst {
                    Register::Local(_) |
                    Register::Call(_) => {
                        lines.push(format!("{indent}{}.append({})", place(dst), py_value(value)));
                    },
                    Register::Return => {
                        lines.push(format!("{indent}ret={}", py_value(value)));
                    },
                    Register::Const(span) => todo!(),
                },
                Bytecode::Pop(src) => match src {
                    Register::Local(_) |
                    Register::Call(_) => {
                        lines.push(format!("{indent}{}.pop()", place(src)));
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
                        lines.push(format!("{indent}s={n}"));
                        lines.push(format!("{indent}continue"));
                    },
                    _ => unreachable!(),
                },
                Bytecode::Intrinsic(intrinsic) => match intrinsic {
                    Intrinsic::IntegerAdd => {
                        lines.push(format!("{indent}ret=c0[-1]+c1[-1]"));
                    },
                    Intrinsic::IntegerEq => {
                        lines.push(format!("{indent}ret=c0[-1]==c1[-1]"));
                    },
                },
                Bytecode::Label(_) => unreachable!(),
                Bytecode::Return => {
                    lines.push(format!("{indent}s=cs[-1]"));
                    lines.push(format!("{indent}continue"));
                },
            }
        }
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
        Register::Const(span) => todo!(),
    }
}

fn peek(r: &Register) -> String {
    match r {
        Register::Local(n @ 0..=9) => format!("l{n}[-1]"),
        Register::Local(n) => format!("lrs[{}][-1]", *n - 10),
        Register::Call(n @ 0..=9) => format!("c{n}[-1]"),
        Register::Call(n) => format!("crs[{}][-1]", *n - 10),
        Register::Return => String::from("ret"),
        Register::Const(span) => todo!(),
    }
}

fn py_value(v: &Const) -> String {
    match v {
        Const::String(s) => format!(
            "{:?}",
            // TODO
            String::from_utf8_lossy(&s.try_unintern_short_string().unwrap_or(b"???".to_vec())),
        ),
        Const::Number(n) => todo!(),
    }
}
