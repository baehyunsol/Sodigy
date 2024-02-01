#![deny(unused_imports)]

pub mod result;
pub mod stages;
pub mod utils;

#[cfg(test)]
mod tests;

use crate::result::CompilerOutput;
use crate::stages::{PathOrRawInput, hir_from_tokens, parse_file};
use crate::utils::{clean_irs, try_make_intermediate_paths};
use sodigy_clap::{CompilerOption, IrStage, SpecialOutput};
use sodigy_endec::Endec;
use sodigy_test::{sodigy_log, LOG_NORMAL};

pub fn run(options: CompilerOption, prev_output: Option<CompilerOutput>) -> CompilerOutput {
    sodigy_log!(LOG_NORMAL, format!("run: enter, input file is `{:?}`", options.input_file));

    let mut compiler_output = prev_output.unwrap_or_default();

    if let Some(sp) = options.do_not_compile_and_do_this {
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
                clean_irs(".", &mut compiler_output);
                compiler_output.dump_to_stdout(format!("cleaning done..."));
            },
        }

        return compiler_output;
    }

    for (is_necessary, is_file, path) in [
        // (is_necessary, is_file, path)
        (options.output_path.is_some(), true, options.output_path.as_ref()),

        // for now, `__sdg_cache__` directories are generated by `generate_path_for_ir` func
        // (options.save_ir, false, `__sdg_cache__` for all directories),

        (options.dump_tokens, true, options.dump_tokens_to.as_ref()),
        (options.dump_hir, true, options.dump_hir_to.as_ref()),
    ] {
        if !is_necessary || path.is_none() {
            continue;
        }

        if let Err(e) = try_make_intermediate_paths(
            is_file, path.unwrap(),
        ) {
            compiler_output.push_error(e);
        }
    }

    let input = if let Some(path) = &options.input_file {
        PathOrRawInput::Path(path)
    } else if let Some(raw_input) = &options.raw_input {
        PathOrRawInput::RawInput(raw_input)
    } else {
        // sodigy_clap guarantees it
        unreachable!()
    };

    let (result, mut compiler_output_) = match options.output_format {
        IrStage::Tokens => {
            let (r, o) = parse_file(
                input,
                Some(compiler_output),
                &options,
            );

            (r.map(|r| Box::new(r) as Box<dyn Endec>), o)
        },
        IrStage::HighIr => {
            let (r, o) = hir_from_tokens(input, Some(compiler_output), &options);

            (r.map(|r| Box::new(r) as Box<dyn Endec>), o)
        },
    };

    if let Some(r) = result {
        if let Some(output_path) = &options.output_path {
            if let Err(e) = r.save_to_file(output_path, None) {
                compiler_output_.push_error(e.into());
            }
        }
    }

    compiler_output_
}

pub const SAVE_IRS_AT: &str = "__sdg_cache__";
pub const COMPILER_HELP_MESSAGE: &str =
"Usage: sodigy [OPTIONS] INPUT

Examples:
    sodigy a.sdg --to tokens -o a.tokens
        It reads `a.sdg` and converts the code into tokens. But it doesn't make an AST.
        It just saves the tokens to `a.tokens`. You can later resume the compilation
        from this stage.

    sodigy a.tokens --to hir -o a.hir
        In the previous example, we paused the compilation before building an AST.
        This option resumes the compilation and generates an HIR.

Options:
    -h, --help                      Display this message
    -v, --version
    -t, --to [tokens|hir]           Specify the type of the output
                                    It tries to infer the output type from the extension of the output.
                                    If the the extension and `-t` doesn't match, `-t` has higher priority.
                                    If there's no other information the default value is hir.
    -o, --output PATH               Write output to <PATH>
    --show-warnings [true|false]    Show warnings messages (default: true)
    --save-ir [true|false]          Save intermediate representations (default: true)
                                    The compiler makes `__sdg_cache__` directory, and save the intermediate
                                    representations in the directory.
    --ignore-saved-ir [true|false]  Disable incremental compilation (default: false)
                                    It still saves intermediate representations when this flag is set.
                                    You have to set `--save-ir false` to not save irs.
                                    TODO: not implemented yet
    --dump-tokens [true|false]      Dump tokens to stdout (default: false)
    --dump-tokens-to PATH           If `dump-tokens` is set, the tokens are dumped to <PATH>
                                    instead of stdout. If `dump-tokens` is not set, it doesn't do anything.
    --dump-hir [true|false]         Dump HIR to stdout (default: false)
    --dump-hir-to PATH              If `dump-hir` is set, the HIR is dumped to <PATH>
                                    instead of stdout. If `dump-hir` is not set, it doesn't do anything.
    --raw-input RAW-INPUT           Compile raw input instead of files.
    --verbose [0|1|2]               Set verbosity (default 1)
                                    Set it to 0 to silence it. Set it to 2 for verbose output.
    --clean                         Remove all the `__sdg_cache__` directories in PWD and its sub directories.
                                    This doesn't remove dumped outputs.
";

pub const MAJOR_VERSION: u8 = 0;
pub const MINOR_VERSION: u8 = 0;
pub const PATCH_VERSION: u8 = 0;
