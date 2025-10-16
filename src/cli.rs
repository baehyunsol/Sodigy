use crate::{Backend, IrKind, Profile};
use ragit_cli::{
    ArgCount,
    ArgParser,
    ArgType,
    Error as CliError,
};

#[derive(Clone, Debug)]
pub enum Command {
    Compile {
        input_path: String,
        input_kind: IrKind,
        intermediate_dir: String,
        output_path: String,
        output_kind: IrKind,
        backend: Backend,
        profile: Profile,
    },
    Interpret {
        bytecode_path: String,
    },
    Help(String),
}

pub fn parse_args(args: &[String]) -> Result<Vec<Command>, CliError> {
    match args.get(1).map(|a| a.as_str()) {
        Some("compile") => {
            let parsed_args = ArgParser::new()
                .optional_arg_flag("--output", ArgType::String)
                .optional_arg_flag("--ir", ArgType::String)
                .optional_arg_flag("--backend", ArgType::enum_(&["c", "rust", "python", "bytecode"]))
                .optional_flag(&["--release", "--test"])
                .alias("-O", "--release")
                .short_flag(&["--output"])
                .args(ArgType::String, ArgCount::Exact(1))  // input path
                .parse(&args, 2)?;

            if parsed_args.show_help() {
                return Ok(vec![Command::Help]);
            }

            let input_path = parsed_args.get_args_exact(1)?[0].to_string();
            let intermediate_dir = parsed_args.arg_flags.get("--ir");
            let output_path = parsed_args.arg_flags.get("--output").map(|p| p.to_string());
            let backend = match parsed_args.arg_flags.get("--backend").map(|f| f.as_str()) {
                Some("c") => Backend::C,
                Some("rust") => Backend::Rust,
                Some("python") => Backend::Python,
                Some("bytecode") => Backend::Bytecode,
                None => Backend::C,  // default
                _ => unreachable!(),
            };
            let profile = match parsed_args.get_flag(0).map(|f| f.as_str()) {
                Some("--release") => Profile::Release,
                Some("--test") => Profile::Test,
                None => Profile::Debug,
                _ => unreachable!(),
            };

            let output_path = match output_path {
                Some(output_path) => output_path,
                None => todo!(),  // out.c | out.rs | out.py | out.sbc
            };

            Ok(vec![Command::Compile {
                input_path,
                input_kind: IrKind::Code,
                intermediate_dir,
                output_path,
                backend,
                profile,
            }])
        },
        Some("compile-hir") => todo!(),
        Some("run") => Ok(vec![
            Command::Compile {},
            Command::Interpret {},
        ]),
        Some(_) => todo!(),
        None => todo!(),
    }
}
