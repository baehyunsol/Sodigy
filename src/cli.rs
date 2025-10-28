use crate::{Backend, IrKind, Profile};
use ragit_cli::{
    ArgCount,
    ArgParser,
    ArgType,
    Error as CliError,
};

#[derive(Clone, Debug)]
pub enum Command {
    InitIrDir {
        intermediate_dir: String,
    },
    Compile {
        input_path: String,
        input_kind: IrKind,
        intermediate_dir: String,
        reuse_ir: bool,

        // These two are for debugging the type-checker.
        // I'll make a CLI option for these, someday.
        emit_irs: bool,
        dump_type_info: bool,

        output_path: FileOrMemory,
        output_kind: IrKind,
        backend: Backend,
        profile: Profile,
    },
    Interpret {
        executable_path: FileOrMemory,

        // It's either `Test` or not.
        // The bytecode will tell you where the tests are, if exist, and where the
        // main function is, if exists. But it won't tell you how to optimize itself.
        profile: Profile,
    },
    Help(String),
}

#[derive(Clone, Debug)]
pub enum FileOrMemory {
    File(String),
    Memory,
}

pub fn parse_args(args: &[String]) -> Result<Vec<Command>, CliError> {
    match args.get(1).map(|a| a.as_str()) {
        Some("compile") => {
            let parsed_args = ArgParser::new()
                .optional_arg_flag("--output", ArgType::String)
                .optional_arg_flag("--ir", ArgType::String)
                .optional_arg_flag("--backend", ArgType::enum_(&["c", "rust", "python", "bytecode"]))
                .optional_flag(&["--reuse-ir"])
                .optional_flag(&["--release", "--test"])
                .alias("-O", "--release")
                .short_flag(&["--output"])
                .args(ArgType::String, ArgCount::Exact(1))  // input path
                .parse(&args, 2)?;

            if parsed_args.show_help() {
                return Ok(vec![Command::Help(String::from("compile"))]);
            }

            let input_path = parsed_args.get_args_exact(1)?[0].to_string();
            let intermediate_dir = parsed_args.arg_flags.get("--ir").map(|p| p.to_string()).unwrap_or_else(|| String::from("__sodigy_cache__"));
            let output_path = parsed_args.arg_flags.get("--output").map(|p| p.to_string());
            let backend = match parsed_args.arg_flags.get("--backend").map(|f| f.as_str()) {
                Some("c") => Backend::C,
                Some("rust") => Backend::Rust,
                Some("python") => Backend::Python,
                Some("bytecode") => Backend::Bytecode,
                None => Backend::Bytecode,  // default
                _ => unreachable!(),
            };
            let reuse_ir = parsed_args.get_flag(0).is_some();

            // Do you see `.as_ref()` and `.map()` below? It's one of the reasons why I'm creating Sodigy.
            let profile = match parsed_args.get_flag(1).as_ref().map(|f| f.as_str()) {
                Some("--release") => Profile::Release,
                Some("--test") => Profile::Test,
                None => Profile::Debug,
                _ => unreachable!(),
            };

            let output_path = match output_path {
                Some(output_path) => output_path,
                None => match backend {
                    Backend::C => "out.c",
                    Backend::Rust => "out.rs",
                    Backend::Python => "out.py",
                    Backend::Bytecode => "out.sbc",
                }.to_string(),
            };

            Ok(vec![
                Command::InitIrDir {
                    intermediate_dir: intermediate_dir.clone(),
                },
                Command::Compile {
                    input_path,
                    input_kind: IrKind::Code,
                    intermediate_dir,
                    reuse_ir,
                    emit_irs: true,
                    dump_type_info: true,
                    output_path: FileOrMemory::File(output_path),
                    output_kind: IrKind::TranspiledCode,
                    backend,
                    profile,
                },
            ])
        },
        Some("compile-hir") => todo!(),
        Some("run") => {
            let parsed_args = ArgParser::new()
                .optional_arg_flag("--ir", ArgType::String)
                .optional_flag(&["--reuse-ir"])
                .optional_flag(&["--release"])
                .alias("-O", "--release")
                .args(ArgType::String, ArgCount::Exact(1))  // input path
                .parse(&args, 2)?;

            let input_path = parsed_args.get_args_exact(1)?[0].to_string();
            let intermediate_dir = parsed_args.arg_flags.get("--ir").map(|p| p.to_string()).unwrap_or_else(|| String::from("__sodigy_cache__"));
            let reuse_ir = parsed_args.get_flag(0).is_some();
            let profile = match parsed_args.get_flag(1).as_ref().map(|f| f.as_str()) {
                Some("--release") => Profile::Release,
                None => Profile::Debug,
                _ => unreachable!(),
            };

            Ok(vec![
                Command::InitIrDir {
                    intermediate_dir: intermediate_dir.clone(),
                },
                Command::Compile {
                    input_path,
                    input_kind: IrKind::Code,
                    intermediate_dir,
                    reuse_ir,
                    emit_irs: true,
                    dump_type_info: true,
                    output_path: FileOrMemory::Memory,
                    output_kind: IrKind::Bytecode,
                    backend: Backend::Bytecode,
                    profile,
                },
                Command::Interpret {
                    executable_path: FileOrMemory::Memory,
                    profile,
                },
            ])
        },
        Some("test") => {
            let parsed_args = ArgParser::new()
                .optional_arg_flag("--ir", ArgType::String)
                .optional_flag(&["--reuse-ir"])
                .args(ArgType::String, ArgCount::Exact(1))  // input path
                .parse(&args, 2)?;

            let input_path = parsed_args.get_args_exact(1)?[0].to_string();
            let intermediate_dir = parsed_args.arg_flags.get("--ir").map(|p| p.to_string()).unwrap_or_else(|| String::from("__sodigy_cache__"));
            let reuse_ir = parsed_args.get_flag(0).is_some();

            Ok(vec![
                Command::InitIrDir {
                    intermediate_dir: intermediate_dir.clone(),
                },
                Command::Compile {
                    input_path,
                    input_kind: IrKind::Code,
                    intermediate_dir,
                    reuse_ir,
                    emit_irs: true,
                    dump_type_info: true,
                    output_path: FileOrMemory::Memory,
                    output_kind: IrKind::Bytecode,
                    backend: Backend::Bytecode,
                    profile: Profile::Test,
                },
                Command::Interpret {
                    executable_path: FileOrMemory::Memory,
                    profile: Profile::Test,
                },
            ])
        },
        Some(_) => todo!(),
        None => todo!(),
    }
}
