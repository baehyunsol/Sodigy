use crate::{Backend, ValidateTokenSpans};
use sodigy_cli::{
    ArgCount,
    ArgParser,
    ArgType,
    Error as CliError,
};
use sodigy_code_gen::Emit;
use sodigy_error::CustomErrorLevel;
use sodigy_object_file::{AssertionFilter, Profile};
use sodigy_optimize::OptimizeLevel;
use std::collections::HashMap;

// If `--bytecode` is set, it reads a bytecode file (from `sodigy build --emit=bytecode`) instead of
// reading `src/`.
#[derive(Debug)]
pub enum CliCommand {
    Build {
        output_path: String,
        emit: Emit,
        bytecode: Option<String>,
        profile: Profile,
        optimize_level: OptimizeLevel,
        custom_error_levels: HashMap<u16, CustomErrorLevel>,
        graceful_shutdown: u32,  // in millis
        validate_token_spans: ValidateTokenSpans,
        check_allocator: bool,
        debug_bytecode: bool,
        jobs: usize,
        color: ColorWhen,
        dump_post_mir_log: bool,
        dump_timings: bool,
    },
    Run {
        bytecode: Option<String>,
        optimize_level: OptimizeLevel,
        backend: Backend,
        custom_error_levels: HashMap<u16, CustomErrorLevel>,
        graceful_shutdown: u32,  // in millis
        validate_token_spans: ValidateTokenSpans,
        check_allocator: bool,
        debug_bytecode: bool,
        jobs: usize,
        color: ColorWhen,
        dump_post_mir_log: bool,
        dump_timings: bool,
    },
    Test {
        bytecode: Option<String>,
        optimize_level: OptimizeLevel,
        backend: Backend,
        custom_error_levels: HashMap<u16, CustomErrorLevel>,
        graceful_shutdown: u32,  // in millis
        std_assertions: bool,
        filters: Option<Vec<AssertionFilter>>,
        validate_token_spans: ValidateTokenSpans,
        check_allocator: bool,
        debug_bytecode: bool,
        jobs: usize,
        color: ColorWhen,
        dump_post_mir_log: bool,
        dump_timings: bool,
    },
    Clean,
    Help {
        command: Option<String>,
        wrong_command: Option<String>,
    },
    Interpret {
        bytecodes_path: String,
        check_allocator: bool,
        debug_bytecode: bool,
    },
    New {
        project_name: String,
    },
}

impl CliCommand {
    pub fn help(command: &str) -> CliCommand {
        CliCommand::Help {
            command: Some(command.to_string()),
            wrong_command: None,
        }
    }

    pub fn all_commands() -> Vec<String> {
        vec![
            String::from("build"),
            String::from("clean"),
            String::from("help"),
            String::from("interpret"),
            String::from("new"),
            String::from("run"),
            String::from("test"),
        ]
    }
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
                .optional_arg_flag("--emit", ArgType::enum_(&["exe", "bytecode", "bytecode-exe", "rust"]))
                .optional_arg_flag("--bytecode", ArgType::String)
                .optional_arg_flag("--filter", ArgType::String)
                .optional_arg_flag("--color", ArgType::enum_(&["auto", "always", "never"]))
                .optional_arg_flag("--jobs", ArgType::integer_between(Some(1), Some(u32::MAX.into())))
                .optional_flag(&["--release"])
                .optional_flag(&["--test"])
                .optional_flag(&["--std-assertions"])
                .optional_flag(&["--dump-post-mir-log"])
                .optional_flag(&["--dump-timings"])
                .flag_with_default(&[
                    "--no-validate-token-spans",
                    "--validate-token-spans",
                    "--validate-std-token-spans",
                    "--validate-lib-token-spans",
                ])
                .optional_flag(&["--debug-bytecode"])
                .optional_flag(&["--check-allocator"])
                .alias("-O", "--release")
                .short_flag(&["--output", "--jobs"])
                .args(ArgType::String, ArgCount::None)
                .parse(args, 2)?;

            if parsed_args.show_help() {
                return Ok(CliCommand::help("build"));
            }

            let output_path = parsed_args.arg_flags.get("--output").map(|p| p.to_string());
            let emit = match parsed_args.arg_flags.get("--emit").map(|f| f.as_str()) {
                Some("exe") => Emit::Exe,
                Some("bytecode") => Emit::ReadableBytecode,
                Some("bytecode-exe") => Emit::ExecutableBytecode,
                Some("rust") => Emit::Rust,
                None => Emit::Exe,  // default
                _ => unreachable!(),
            };
            let bytecode = parsed_args.arg_flags.get("--bytecode").map(|b| b.to_string());
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

            let profile = match (
                parsed_args.get_flag(1).is_some(),  // --test
                parsed_args.get_flag(2).is_some(),  // --std-assertions
                parsed_args.arg_flags.get("--filter"),
            ) {
                (false, true, _) | (false, _, Some(_)) => todo!(),  // a cli error
                (false, _, _) => Profile::Run,
                (true, std_assertions, filter) => Profile::Test {
                    std_assertions,
                    filters: filter.map(|f| todo!()),
                },
            };

            let dump_post_mir_log = parsed_args.get_flag(3).is_some();
            let dump_timings = parsed_args.get_flag(4).is_some();

            let validate_token_spans = match parsed_args.get_flag(5).as_ref().map(|s| s.as_str()) {
                Some("--no-validate-token-spans") => ValidateTokenSpans::Never,
                Some("--validate-token-spans") => ValidateTokenSpans::Always,
                Some("--validate-std-token-spans") => ValidateTokenSpans::OnlyStd,
                Some("--validate-lib-token-spans") => ValidateTokenSpans::ExceptStd,
                _ => unreachable!(),
            };

            let debug_bytecode = match (emit, parsed_args.get_flag(6).is_some()) {
                (Emit::Exe | Emit::Rust, true) => true,
                (Emit::ReadableBytecode | Emit::ExecutableBytecode, true) => {
                    // This is a cli error. You can set `--debug-bytecode` flag only if the emit option is `rust` or `exe`,
                    // because the interpreter can run an object file with/without the debug session.
                    // But there's no way I can construct such CliError...
                    todo!()
                },
                (_, false) => false,
            };
            let check_allocator = parsed_args.get_flag(7).is_some();

            let output_path = match output_path {
                Some(output_path) => output_path,

                // default value
                None => match emit {
                    Emit::Exe => if cfg!(target_os = "windows") { "out.exe" } else { "out" },
                    Emit::ReadableBytecode => "out.sdgb",
                    Emit::ExecutableBytecode => "out.sdge",
                    Emit::Rust => "out.rs",
                }.to_string(),
            };

            Ok(CliCommand::Build {
                output_path,
                emit,
                bytecode,
                profile,
                optimize_level,
                custom_error_levels: HashMap::new(),  // TODO: make it configurable
                graceful_shutdown: 300,  // TODO: make it configurable
                validate_token_spans,
                check_allocator,
                debug_bytecode,
                jobs,
                color,
                dump_post_mir_log,
                dump_timings,
            })
        },
        Some("clean") => {
            let parsed_args = ArgParser::new()
                .args(ArgType::String, ArgCount::None)
                .parse(args, 2)?;

            if parsed_args.show_help() {
                return Ok(CliCommand::help("clean"));
            }

            Ok(CliCommand::Clean)
        },
        Some("help") => {
            let parsed_args = ArgParser::new()
                .args(ArgType::String, ArgCount::Leq(1))
                .parse(args, 2)?;

            if parsed_args.show_help() {
                return Ok(CliCommand::help("help"));
            }

            Ok(CliCommand::Help {
                command: parsed_args.get_args().get(0).map(|s| s.to_string()),
                wrong_command: None,
            })
        },
        Some("interpret") => {
            let parsed_args = ArgParser::new()
                .optional_flag(&["--debug-bytecode"])
                .optional_flag(&["--check-allocator"])
                .args(ArgType::String, ArgCount::Exact(1))  // bytecodes path
                .parse(args, 2)?;

            if parsed_args.show_help() {
                return Ok(CliCommand::help("interpret"));
            }

            let debug_bytecode = parsed_args.get_flag(0).is_some();
            let check_allocator = parsed_args.get_flag(1).is_some();
            let bytecodes_path = parsed_args.get_args_exact(1)?[0].to_string();

            Ok(CliCommand::Interpret {
                bytecodes_path,
                check_allocator,
                debug_bytecode,
            })
        },
        Some("new") => {
            let parsed_args = ArgParser::new()
                .args(ArgType::String, ArgCount::Exact(1))  // project name
                .parse(args, 2)?;

            if parsed_args.show_help() {
                return Ok(CliCommand::help("new"));
            }

            let project_name = parsed_args.get_args_exact(1)?[0].to_string();

            Ok(CliCommand::New { project_name })
        },
        Some("run") => {
            let parsed_args = ArgParser::new()
                .optional_arg_flag("--bytecode", ArgType::String)
                .optional_arg_flag("--backend", ArgType::enum_(&["native", "interpret", "mir-interpret"]))
                .optional_arg_flag("--color", ArgType::enum_(&["auto", "always", "never"]))
                .optional_arg_flag("--jobs", ArgType::integer_between(Some(1), Some(u32::MAX.into())))
                .optional_flag(&["--release"])
                .optional_flag(&["--dump-post-mir-log"])
                .optional_flag(&["--dump-timings"])
                .flag_with_default(&[
                    "--no-validate-token-spans",
                    "--validate-token-spans",
                    "--validate-std-token-spans",
                    "--validate-lib-token-spans",
                ])
                .optional_flag(&["--debug-bytecode"])
                .optional_flag(&["--check-allocator"])
                .alias("-O", "--release")
                .short_flag(&["--jobs"])
                .args(ArgType::String, ArgCount::None)
                .parse(args, 2)?;

            if parsed_args.show_help() {
                return Ok(CliCommand::help("run"));
            }

            let optimize_level = match parsed_args.get_flag(0).as_ref().map(|f| f.as_str()) {
                Some("--release") => OptimizeLevel::Mild,
                None => OptimizeLevel::None,
                _ => unreachable!(),
            };
            let bytecode = parsed_args.arg_flags.get("--bytecode").map(|b| b.to_string());
            let backend = match parsed_args.arg_flags.get("--backend").map(|f| f.as_str()) {
                Some("native") => Backend::Native,
                Some("interpret") => Backend::Interpret,
                Some("mir-interpret") => Backend::MirInterpret,

                // default value
                None => if optimize_level == OptimizeLevel::None {
                    Backend::Interpret
                } else {
                    Backend::Native
                },

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
            let dump_post_mir_log = parsed_args.get_flag(1).is_some();
            let dump_timings = parsed_args.get_flag(2).is_some();

            let validate_token_spans = match parsed_args.get_flag(3).as_ref().map(|s| s.as_str()) {
                Some("--no-validate-token-spans") => ValidateTokenSpans::Never,
                Some("--validate-token-spans") => ValidateTokenSpans::Always,
                Some("--validate-std-token-spans") => ValidateTokenSpans::OnlyStd,
                Some("--validate-lib-token-spans") => ValidateTokenSpans::ExceptStd,
                _ => unreachable!(),
            };

            let debug_bytecode = parsed_args.get_flag(4).is_some();
            let check_allocator = parsed_args.get_flag(5).is_some();

            Ok(CliCommand::Run {
                bytecode,
                optimize_level,
                backend,
                custom_error_levels: HashMap::new(),  // TODO: make it configurable
                graceful_shutdown: 300,  // TODO: make it configurable
                validate_token_spans,
                check_allocator,
                debug_bytecode,
                jobs,
                color,
                dump_post_mir_log,
                dump_timings,
            })
        },
        Some("test") => {
            let parsed_args = ArgParser::new()
                .optional_arg_flag("--bytecode", ArgType::String)
                .optional_arg_flag("--backend", ArgType::enum_(&["native", "interpret", "mir-interpret"]))
                .optional_arg_flag("--filter", ArgType::String)
                .optional_arg_flag("--color", ArgType::enum_(&["auto", "always", "never"]))
                .optional_arg_flag("--jobs", ArgType::integer_between(Some(1), Some(u32::MAX.into())))
                .optional_flag(&["--release"])
                .optional_flag(&["--std-assertions"])
                .optional_flag(&["--dump-post-mir-log"])
                .optional_flag(&["--dump-timings"])
                .flag_with_default(&[
                    "--no-validate-token-spans",
                    "--validate-token-spans",
                    "--validate-std-token-spans",
                    "--validate-lib-token-spans",
                ])
                .optional_flag(&["--debug-bytecode"])
                .optional_flag(&["--check-allocator"])
                .alias("-O", "--release")
                .short_flag(&["--jobs"])
                .args(ArgType::String, ArgCount::None)
                .parse(args, 2)?;

            if parsed_args.show_help() {
                return Ok(CliCommand::help("test"));
            }

            let optimize_level = match parsed_args.get_flag(0).as_ref().map(|f| f.as_str()) {
                Some("--release") => OptimizeLevel::Mild,
                None => OptimizeLevel::None,
                _ => unreachable!(),
            };
            let bytecode = parsed_args.arg_flags.get("--bytecode").map(|b| b.to_string());
            let backend = match parsed_args.arg_flags.get("--backend").map(|f| f.as_str()) {
                Some("native") => Backend::Native,
                Some("interpret") => Backend::Interpret,
                Some("mir-interpret") => Backend::MirInterpret,

                // default value
                None => if optimize_level == OptimizeLevel::None {
                    Backend::Interpret
                } else {
                    Backend::Native
                },

                _ => unreachable!(),
            };
            let filter = parsed_args.arg_flags.get("--filter").map(|f| todo!());
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
            let std_assertions = parsed_args.get_flag(1).is_some();
            let dump_post_mir_log = parsed_args.get_flag(2).is_some();
            let dump_timings = parsed_args.get_flag(3).is_some();

            let validate_token_spans = match parsed_args.get_flag(4).as_ref().map(|s| s.as_str()) {
                Some("--no-validate-token-spans") => ValidateTokenSpans::Never,
                Some("--validate-token-spans") => ValidateTokenSpans::Always,
                Some("--validate-std-token-spans") => ValidateTokenSpans::OnlyStd,
                Some("--validate-lib-token-spans") => ValidateTokenSpans::ExceptStd,
                _ => unreachable!(),
            };
            let debug_bytecode = parsed_args.get_flag(5).is_some();
            let check_allocator = parsed_args.get_flag(6).is_some();

            Ok(CliCommand::Test {
                bytecode,
                optimize_level,
                backend,
                custom_error_levels: HashMap::new(),  // TODO: make it configurable
                graceful_shutdown: 300,  // TODO: make it configurable
                std_assertions,
                filters: filter,
                validate_token_spans,
                check_allocator,
                debug_bytecode,
                jobs,
                color,
                dump_post_mir_log,
                dump_timings,
            })
        },
        Some(command) => Ok(CliCommand::Help {
            command: None,
            wrong_command: Some(command.to_string()),
        }),
        None => Ok(CliCommand::Help {
            command: None,
            wrong_command: None,
        }),
    }
}
