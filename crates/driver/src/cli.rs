use crate::{Profile, ValidateTokenSpans};
use sodigy_cli::{
    ArgCount,
    ArgParser,
    ArgType,
    Error as CliError,
};
use sodigy_code_gen::Backend;
use sodigy_error::CustomErrorLevel;
use sodigy_optimize::OptimizeLevel;
use std::collections::HashMap;

#[derive(Debug)]
pub enum CliCommand {
    Build {
        output_path: String,
        backend: Backend,
        optimize_level: OptimizeLevel,
        import_std: bool,
        custom_error_levels: HashMap<u16, CustomErrorLevel>,
        profile: Profile,
        emit_irs: bool,
        graceful_shutdown: u32,  // in millis
        validate_token_spans: ValidateTokenSpans,
        jobs: usize,
        color: ColorWhen,
        dump_post_mir_log: bool,
    },
    Run {
        optimize_level: OptimizeLevel,
        import_std: bool,
        custom_error_levels: HashMap<u16, CustomErrorLevel>,
        emit_irs: bool,
        graceful_shutdown: u32,  // in millis
        validate_token_spans: ValidateTokenSpans,
        jobs: usize,
        color: ColorWhen,
        dump_post_mir_log: bool,
    },
    Test {
        optimize_level: OptimizeLevel,
        import_std: bool,
        custom_error_levels: HashMap<u16, CustomErrorLevel>,
        emit_irs: bool,
        graceful_shutdown: u32,  // in millis
        validate_token_spans: ValidateTokenSpans,
        jobs: usize,
        color: ColorWhen,
        dump_post_mir_log: bool,
    },
    Clean,
    Help(String),
    Interpret {
        bytecodes_path: String,
        profile: Profile,
    },
    New {
        project_name: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ColorWhen {
    Auto,
    Always,
    Never,
}

pub fn parse_args(args: &[String]) -> Result<CliCommand, CliError> {
    match args.get(1).map(|a| a.as_str()) {
        Some("build") => {
            let parsed_args = ArgParser::new()
                .optional_arg_flag("--output", ArgType::String)
                .optional_arg_flag("--backend", ArgType::enum_(&["c", "rust", "python", "bytecode"]))
                .optional_arg_flag("--color", ArgType::enum_(&["auto", "always", "never"]))
                .optional_arg_flag("--jobs", ArgType::integer_between(Some(1), Some(u32::MAX.into())))
                .optional_flag(&["--release"])
                .optional_flag(&["--test"])
                .optional_flag(&["--emit-irs"])
                .optional_flag(&["--no-std"])
                .optional_flag(&["--dump-post-mir-log"])
                .flag_with_default(&[
                    "--no-validate-token-spans",
                    "--validate-token-spans",
                    "--validate-std-token-spans",
                    "--validate-lib-token-spans",
                ])
                .alias("-O", "--release")
                .short_flag(&["--output", "--jobs"])
                .args(ArgType::String, ArgCount::None)
                .parse(args, 2)?;

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
            let color = match parsed_args.arg_flags.get("--color").map(|f| f.as_str()) {
                Some("auto") => ColorWhen::Auto,
                Some("always") => ColorWhen::Always,
                Some("never") => ColorWhen::Never,
                None => ColorWhen::Auto,  // default
                _ => unreachable!(),
            };
            let jobs = parsed_args.arg_flags.get("--jobs").map(
                |n| n.parse::<usize>().unwrap()
            ).unwrap_or_else(
                || std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4)
            );

            // Do you see `.as_ref()` and `.map()` below? It's one of the reasons why I'm creating Sodigy.
            let optimize_level = match parsed_args.get_flag(0).as_ref().map(|f| f.as_str()) {
                Some("--release") => OptimizeLevel::Mild,
                None => OptimizeLevel::None,
                _ => unreachable!(),
            };

            let profile = match parsed_args.get_flag(1).as_ref().map(|f| f.as_str()) {
                Some("--test") => Profile::Test,
                None => Profile::Script,
                _ => unreachable!(),
            };

            let emit_irs = parsed_args.get_flag(2).is_some();
            let import_std = parsed_args.get_flag(3).is_none();
            let dump_post_mir_log = parsed_args.get_flag(4).is_some();

            let validate_token_spans = match parsed_args.get_flag(5).as_ref().map(|s| s.as_str()) {
                Some("--no-validate-token-spans") => ValidateTokenSpans::Never,
                Some("--validate-token-spans") => ValidateTokenSpans::Always,
                Some("--validate-std-token-spans") => ValidateTokenSpans::OnlyStd,
                Some("--validate-lib-token-spans") => ValidateTokenSpans::ExceptStd,
                _ => unreachable!(),
            };

            let output_path = match output_path {
                Some(output_path) => output_path,
                None => String::from("out.sdgbc"),
            };

            Ok(CliCommand::Build {
                output_path,
                backend,
                optimize_level,
                import_std,
                custom_error_levels: HashMap::new(),  // TODO: make it configurable
                graceful_shutdown: 300,  // TODO: make it configurable
                validate_token_spans,
                profile,
                emit_irs,
                jobs,
                color,
                dump_post_mir_log,
            })
        },
        Some("clean") => {
            let parsed_args = ArgParser::new()
                .args(ArgType::String, ArgCount::None)
                .parse(args, 2)?;

            if parsed_args.show_help() {
                return Ok(CliCommand::Help(String::from("clean")));
            }

            Ok(CliCommand::Clean)
        },
        Some("help") => {
            let parsed_args = ArgParser::new()
                .args(ArgType::String, ArgCount::Exact(1))
                .parse(args, 2)?;

            if parsed_args.show_help() {
                return Ok(CliCommand::Help(String::from("clean")));
            }

            let help = parsed_args.get_args_exact(1)?[0].to_string();

            Ok(CliCommand::Help(help))
        },
        Some("interpret") => {
            let parsed_args = ArgParser::new()
                .args(ArgType::String, ArgCount::Exact(1))  // bytecodes path
                .parse(args, 2)?;

            if parsed_args.show_help() {
                return Ok(CliCommand::Help(String::from("interpret")));
            }

            let bytecodes_path = parsed_args.get_args_exact(1)?[0].to_string();

            Ok(CliCommand::Interpret {
                bytecodes_path,

                // TODO: make it configurable
                profile: Profile::Test,
            })
        },
        Some("new") => {
            let parsed_args = ArgParser::new()
                .args(ArgType::String, ArgCount::Exact(1))  // project name
                .parse(args, 2)?;

            if parsed_args.show_help() {
                return Ok(CliCommand::Help(String::from("new")));
            }

            let project_name = parsed_args.get_args_exact(1)?[0].to_string();

            Ok(CliCommand::New { project_name })
        },
        Some("run") => {
            let parsed_args = ArgParser::new()
                .optional_arg_flag("--color", ArgType::enum_(&["auto", "always", "never"]))
                .optional_arg_flag("--jobs", ArgType::integer_between(Some(1), Some(u32::MAX.into())))
                .optional_flag(&["--release"])
                .optional_flag(&["--emit-irs"])
                .optional_flag(&["--no-std"])
                .optional_flag(&["--dump-post-mir-log"])
                .flag_with_default(&[
                    "--no-validate-token-spans",
                    "--validate-token-spans",
                    "--validate-std-token-spans",
                    "--validate-lib-token-spans",
                ])
                .alias("-O", "--release")
                .short_flag(&["--jobs"])
                .args(ArgType::String, ArgCount::None)
                .parse(args, 2)?;

            if parsed_args.show_help() {
                return Ok(CliCommand::Help(String::from("run")));
            }

            let optimize_level = match parsed_args.get_flag(0).as_ref().map(|f| f.as_str()) {
                Some("--release") => OptimizeLevel::Mild,
                None => OptimizeLevel::None,
                _ => unreachable!(),
            };
            let color = match parsed_args.arg_flags.get("--color").map(|f| f.as_str()) {
                Some("auto") => ColorWhen::Auto,
                Some("always") => ColorWhen::Always,
                Some("never") => ColorWhen::Never,
                None => ColorWhen::Auto,  // default
                _ => unreachable!(),
            };
            let jobs = parsed_args.arg_flags.get("--jobs").map(
                |n| n.parse::<usize>().unwrap()
            ).unwrap_or_else(
                || std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4)
            );
            let emit_irs = parsed_args.get_flag(1).is_some();
            let import_std = parsed_args.get_flag(2).is_none();
            let dump_post_mir_log = parsed_args.get_flag(3).is_some();

            let validate_token_spans = match parsed_args.get_flag(4).as_ref().map(|s| s.as_str()) {
                Some("--no-validate-token-spans") => ValidateTokenSpans::Never,
                Some("--validate-token-spans") => ValidateTokenSpans::Always,
                Some("--validate-std-token-spans") => ValidateTokenSpans::OnlyStd,
                Some("--validate-lib-token-spans") => ValidateTokenSpans::ExceptStd,
                _ => unreachable!(),
            };

            Ok(CliCommand::Run {
                optimize_level,
                import_std,
                custom_error_levels: HashMap::new(),  // TODO: make it configurable
                graceful_shutdown: 300,  // TODO: make it configurable
                validate_token_spans,
                emit_irs,
                jobs,
                color,
                dump_post_mir_log,
            })
        },
        Some("test") => {
            let parsed_args = ArgParser::new()
                .optional_arg_flag("--color", ArgType::enum_(&["auto", "always", "never"]))
                .optional_arg_flag("--jobs", ArgType::integer_between(Some(1), Some(u32::MAX.into())))
                .optional_flag(&["--release"])
                .optional_flag(&["--emit-irs"])
                .optional_flag(&["--no-std"])
                .optional_flag(&["--dump-post-mir-log"])
                .flag_with_default(&[
                    "--no-validate-token-spans",
                    "--validate-token-spans",
                    "--validate-std-token-spans",
                    "--validate-lib-token-spans",
                ])
                .alias("-O", "--release")
                .short_flag(&["--jobs"])
                .args(ArgType::String, ArgCount::None)
                .parse(args, 2)?;

            if parsed_args.show_help() {
                return Ok(CliCommand::Help(String::from("test")));
            }

            let optimize_level = match parsed_args.get_flag(0).as_ref().map(|f| f.as_str()) {
                Some("--release") => OptimizeLevel::Mild,
                None => OptimizeLevel::None,
                _ => unreachable!(),
            };
            let color = match parsed_args.arg_flags.get("--color").map(|f| f.as_str()) {
                Some("auto") => ColorWhen::Auto,
                Some("always") => ColorWhen::Always,
                Some("never") => ColorWhen::Never,
                None => ColorWhen::Auto,  // default
                _ => unreachable!(),
            };
            let jobs = parsed_args.arg_flags.get("--jobs").map(
                |n| n.parse::<usize>().unwrap()
            ).unwrap_or_else(
                || std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4)
            );
            let emit_irs = parsed_args.get_flag(1).is_some();
            let import_std = parsed_args.get_flag(2).is_none();
            let dump_post_mir_log = parsed_args.get_flag(3).is_some();

            let validate_token_spans = match parsed_args.get_flag(4).as_ref().map(|s| s.as_str()) {
                Some("--no-validate-token-spans") => ValidateTokenSpans::Never,
                Some("--validate-token-spans") => ValidateTokenSpans::Always,
                Some("--validate-std-token-spans") => ValidateTokenSpans::OnlyStd,
                Some("--validate-lib-token-spans") => ValidateTokenSpans::ExceptStd,
                _ => unreachable!(),
            };

            Ok(CliCommand::Test {
                optimize_level,
                import_std,
                custom_error_levels: HashMap::new(),  // TODO: make it configurable
                graceful_shutdown: 300,  // TODO: make it configurable
                validate_token_spans,
                emit_irs,
                jobs,
                color,
                dump_post_mir_log,
            })
        },
        Some(_) => todo!(),
        None => todo!(),
    }
}
