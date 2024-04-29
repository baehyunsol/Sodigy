#![deny(unused_imports)]

mod error;
mod multi;
pub mod stages;
pub mod utils;

#[cfg(test)]
mod tests;

use crate::stages::{
    PathOrRawInput,
    construct_binary,
    construct_hir,
    construct_mir,
};
pub use crate::utils::clean_irs;
use log::info;
use sodigy_config::{CompilerOption, CompilerOutputFormat, SpecialOutput};
use sodigy_files::{write_bytes, WriteMode};
use sodigy_output::CompilerOutput;

pub fn run(options: CompilerOption) -> CompilerOutput {
    info!("sodigy::run()");

    if let Some(sp) = options.do_not_compile_and_do_this {
        let mut compiler_output = CompilerOutput::new();

        match sp {
            SpecialOutput::HelpMessage => {
                compiler_output.dump_to_stdout(format!("{COMPILER_HELP_MESSAGE}"));
                compiler_output.show_overall_result = false;
            },
            SpecialOutput::VersionInfo => {
                compiler_output.dump_to_stdout(format!("sodigy {MAJOR_VERSION}.{MINOR_VERSION}.{PATCH_VERSION}"));
                compiler_output.show_overall_result = false;
            },
            SpecialOutput::CleanIrs => {
                let mut count = 0;
                clean_irs(".", &mut compiler_output, &mut count);
                compiler_output.dump_to_stdout(format!(
                    "cleaning done: removed {count} dir{}",
                    if count > 1 { "s" } else { "" },
                ));
            },
        }

        return compiler_output;
    }

    let input = if let Some(path) = &options.input_file {
        PathOrRawInput::Path(path)
    } else if let Some(raw_input) = &options.raw_input {
        PathOrRawInput::RawInput(raw_input)
    } else {
        // sodigy_clap guarantees it
        unreachable!()
    };

    let mut output = match &options.output {
        CompilerOutputFormat::HighIr => {
            let (session, mut output) = construct_hir(input, &options, true /* is_root */);

            if let Some(session) = session {
                output.collect_errors_and_warnings_from_session(&session);
            }

            output
        },
        CompilerOutputFormat::MidIr => {
            let (session, mut output) = construct_mir(input, &options);

            if let Some(session) = session {
                output.collect_errors_and_warnings_from_session(&session);
            }

            output
        },
        CompilerOutputFormat::Path(_)
        | CompilerOutputFormat::None => {
            let (result, mut output) = construct_binary(input, &options);

            if let Some(path) = options.output.try_unwrap_path() {
                if let Some(binary) = result {
                    if let Err(e) = write_bytes(path, &binary, WriteMode::CreateOrTruncate) {
                        output.push_error(e.into());
                    }
                }
            }

            output
        },
    };

    output.show_overall_result = true;

    output
}

pub const DEPENDENCIES_AT: &str = "sodigy.json";
pub const SAVE_IRS_AT: &str = "__sdg_cache__";
pub const COMPILER_HELP_MESSAGE: &str =
"Usage: sodigy [OPTIONS] INPUT

Options:
    -h, --help                      Display this message
    -v, --version
    -o, --output PATH               Write output to <PATH>
    --stop-at [hir|mir]             Stop compilation at [hir|mir] stage and don't write output
                                    If `--output` and `--stop-at` are both set, this flag is ignored.
                                    The intermediate representations of [hir|mir] is saved at `__sdg_cache__`.
    --show-warnings [true|false]    Show warnings messages (default: true)
    --save-ir [true|false]          Save intermediate representations (default: true)
                                    The compiler makes `__sdg_cache__` directory, and save the intermediate
                                    representations in the directory.
    --ignore-saved-ir [true|false]  Disable incremental compilation (default: false)
                                    It still saves intermediate representations when this flag is set.
                                    You have to set `--save-ir false` to not save irs.
                                    TODO: not implemented yet
    --dump-hir-to PATH              Dumps the hir session to <PATH> as a json file. If PATH is `STDOUT`, it dumps the
                                    result to stdout. If it's compiled from cached data, `--dump-hir-to` may not work.
                                    If it does not work, try `./sodigy --clean` and compile again. If you're compiling
                                    multiple files, it only dumps the hir of the root file (one that's fed to the cli argument).
    --dump-mir-to PATH              Dumps the mir session to <PATH> as a json file. If PATH is `STDOUT`, it dumps the
                                    result to stdout. If it's compiled from cached data, `--dump-mir-to` may not work.
                                    If it does not work, try `./sodigy --clean` and compile again.
    --raw-input RAW-INPUT           Compile raw input instead of files.
    --verbose [0|1|2]               Set verbosity (default 1)
                                    Set it to 0 to silence it. Set it to 2 for verbose output.
    -w, --num-workers INT           Number of parallel workers (default: the number of CPUs)
    --clean                         Remove all the `__sdg_cache__` directories in PWD and its sub directories.
                                    This doesn't remove dumped outputs.
";

pub const MAJOR_VERSION: u8 = 0;
pub const MINOR_VERSION: u8 = 0;
pub const PATCH_VERSION: u8 = 0;
