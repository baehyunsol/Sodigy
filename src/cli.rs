use crate::{Backend, Optimization, Profile};
use ragit_cli::{
    ArgCount,
    ArgParser,
    ArgType,
    Error as CliError,
};

#[derive(Debug)]
pub enum CliCommand {
    Build {
        output_path: String,
        backend: Backend,
        optimization: Optimization,
        import_std: bool,
        profile: Profile,
        emit_irs: bool,
        jobs: u32,
    },
    Clean,
    Help(String),
    Interpret {
        bytecodes_path: String,
    },
    New {
        project_name: String,
    },
    Run {
        optimization: Optimization,
        import_std: bool,
        jobs: u32,
    },
    Test {
        optimization: Optimization,
        import_std: bool,
        jobs: u32,
    },
}

pub fn parse_args(args: &[String]) -> Result<CliCommand, CliError> {
    match args.get(1).map(|a| a.as_str()) {
        Some("build") => {
            let parsed_args = ArgParser::new()
                .optional_arg_flag("--output", ArgType::String)
                .optional_arg_flag("--backend", ArgType::enum_(&["c", "rust", "python", "bytecode"]))
                .optional_arg_flag("--jobs", ArgType::integer_between(Some(1), Some(u32::MAX.into())))
                .optional_flag(&["--release"])
                .optional_flag(&["--test"])
                .optional_flag(&["--emit-irs"])
                .optional_flag(&["--no-std"])
                .alias("-O", "--release")
                .short_flag(&["--output", "--jobs"])
                .args(ArgType::String, ArgCount::None)
                .parse(&args, 2)?;

            if parsed_args.show_help() {
                return Ok(CliCommand::Help(String::from("build")));
            }

            let output_path = parsed_args.arg_flags.get("--output").map(|p| p.to_string());
            let backend = match parsed_args.arg_flags.get("--backend").map(|f| f.as_str()) {
                Some("c") => Backend::C,
                Some("rust") => Backend::Rust,
                Some("python") => Backend::Python,
                Some("bytecode") => Backend::Bytecode,
                None => Backend::Bytecode,  // default
                _ => unreachable!(),
            };
            let jobs = parsed_args.arg_flags.get("--jobs").map(|n| n.parse::<u32>().unwrap()).unwrap_or(4);

            // Do you see `.as_ref()` and `.map()` below? It's one of the reasons why I'm creating Sodigy.
            let optimization = match parsed_args.get_flag(0).as_ref().map(|f| f.as_str()) {
                Some("--release") => Optimization::Mild,
                None => Optimization::None,
                _ => unreachable!(),
            };

            let profile = match parsed_args.get_flag(1).as_ref().map(|f| f.as_str()) {
                Some("--test") => Profile::Test,
                None => Profile::Script,
                _ => unreachable!(),
            };

            let emit_irs = parsed_args.get_flag(2).is_some();
            let import_std = !parsed_args.get_flag(3).is_some();

            let output_path = match output_path {
                Some(output_path) => output_path,
                None => match backend {
                    Backend::C => "out.c",
                    Backend::Rust => "out.rs",
                    Backend::Python => "out.py",
                    Backend::Bytecode => "out.sdgbc",
                }.to_string(),
            };

            Ok(CliCommand::Build {
                output_path,
                backend,
                optimization,
                import_std,
                profile,
                emit_irs,
                jobs,
            })
        },
        Some("clean") => {
            let parsed_args = ArgParser::new()
                .args(ArgType::String, ArgCount::None)
                .parse(&args, 2)?;

            if parsed_args.show_help() {
                return Ok(CliCommand::Help(String::from("clean")));
            }

            Ok(CliCommand::Clean)
        },
        Some("help") => {
            let parsed_args = ArgParser::new()
                .args(ArgType::String, ArgCount::Exact(1))
                .parse(&args, 2)?;

            if parsed_args.show_help() {
                return Ok(CliCommand::Help(String::from("clean")));
            }

            let help = parsed_args.get_args_exact(1)?[0].to_string();

            Ok(CliCommand::Help(help))
        },
        Some("interpret") => {
            let parsed_args = ArgParser::new()
                .args(ArgType::String, ArgCount::Exact(1))  // bytecodes path
                .parse(&args, 2)?;

            if parsed_args.show_help() {
                return Ok(CliCommand::Help(String::from("interpret")));
            }

            let bytecodes_path = parsed_args.get_args_exact(1)?[0].to_string();

            Ok(CliCommand::Interpret { bytecodes_path })
        },
        Some("new") => {
            let parsed_args = ArgParser::new()
                .args(ArgType::String, ArgCount::Exact(1))  // project name
                .parse(&args, 2)?;

            if parsed_args.show_help() {
                return Ok(CliCommand::Help(String::from("new")));
            }

            let project_name = parsed_args.get_args_exact(1)?[0].to_string();

            Ok(CliCommand::New { project_name })
        },
        Some("run") => {
            let parsed_args = ArgParser::new()
                .optional_arg_flag("--jobs", ArgType::integer_between(Some(1), Some(u32::MAX.into())))
                .optional_flag(&["--release"])
                .optional_flag(&["--no-std"])
                .alias("-O", "--release")
                .short_flag(&["--jobs"])
                .args(ArgType::String, ArgCount::None)
                .parse(&args, 2)?;

            if parsed_args.show_help() {
                return Ok(CliCommand::Help(String::from("run")));
            }

            let optimization = match parsed_args.get_flag(0).as_ref().map(|f| f.as_str()) {
                Some("--release") => Optimization::Mild,
                None => Optimization::None,
                _ => unreachable!(),
            };
            let jobs = parsed_args.arg_flags.get("--jobs").map(|n| n.parse::<u32>().unwrap()).unwrap_or(4);
            let import_std = !parsed_args.get_flag(1).is_some();

            Ok(CliCommand::Run {
                optimization,
                import_std,
                jobs,
            })
        },
        Some("test") => {
            let parsed_args = ArgParser::new()
                .optional_arg_flag("--jobs", ArgType::integer_between(Some(1), Some(u32::MAX.into())))
                .optional_flag(&["--release"])
                .optional_flag(&["--no-std"])
                .alias("-O", "--release")
                .short_flag(&["--jobs"])
                .args(ArgType::String, ArgCount::None)
                .parse(&args, 2)?;

            if parsed_args.show_help() {
                return Ok(CliCommand::Help(String::from("test")));
            }

            let optimization = match parsed_args.get_flag(0).as_ref().map(|f| f.as_str()) {
                Some("--release") => Optimization::Mild,
                None => Optimization::None,
                _ => unreachable!(),
            };
            let jobs = parsed_args.arg_flags.get("--jobs").map(|n| n.parse::<u32>().unwrap()).unwrap_or(4);
            let import_std = !parsed_args.get_flag(1).is_some();

            Ok(CliCommand::Test {
                optimization,
                import_std,
                jobs,
            })
        },
        Some(_) => todo!(),
        None => todo!(),
    }
}
