#![deny(unused_imports)]

mod fmt;
pub mod stages;

#[cfg(test)]
mod tests;

use crate::stages::{
    PathOrRawInput,
    construct_hir,
    construct_mir,
};
use log::info;
use sodigy_config::{CompilerOption, CompilerOutputFormat, SpecialOutput};
use sodigy_endec::Endec;
use sodigy_output::CompilerOutput;
use sodigy_session::SodigySession;

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
        }

        return compiler_output;
    }

    let input = if let Some(path) = &options.input_path {
        PathOrRawInput::Path(path)
    } else if let Some(raw_input) = &options.raw_input {
        PathOrRawInput::RawInput(raw_input)
    } else {
        // sodigy_clap guarantees it
        unreachable!()
    };

    let mut output = match &options.output_format {
        CompilerOutputFormat::Hir => {
            let (session, mut output) = construct_hir(input, &options);

            if let Some(session) = session {
                output.collect_errors_and_warnings_from_session(&session);

                if !session.has_error() {
                    let save_hir_at = options.output_path.as_ref().map(
                        |path| path.to_string()
                    ).unwrap_or_else(
                        || options.output_format.create_output_path()
                    );

                    if let Err(e) = session.save_to_file(&save_hir_at) {
                        output.push_error(e.into());
                    }
                }
            }

            output
        },
        CompilerOutputFormat::Mir
        | CompilerOutputFormat::Binary => {  // NOTE: binary pass is not implemented yet
            let (session, mut output) = construct_mir(input, &options);

            if let Some(session) = session {
                output.collect_errors_and_warnings_from_session(&session);
            }

            // TODO: if it's mir, save it
            //       if it's binary... then what?

            output
        },
        CompilerOutputFormat::None => unreachable!(),  // TODO: what's this format for?
    };

    output.show_overall_result = true;

    output
}

pub const DEPENDENCIES_AT: &str = "sodigy.json";
pub const COMPILER_HELP_MESSAGE: &str =
"Usage: sodigy [OPTIONS] INPUT

Options:
    -h, --help                      Display this message
    -v, --version
    -o, --output PATH               Write output to <PATH>
    -H, --hir                       Generate hir from <INPUT> and write the result to output path
    -M, --mir                       Generate mir from <INPUT> and write the result to output path. The input must be
                                    an hir file, which is generated by the `-H` option.
    -L NAME=PATH                    Specify a path of hir of a library. The hir file is generated by the `-H` option.
    --show-warnings                 Show warning messages (default: on)
    --hide-warnings                 Hide warning messages (default: off)
    --raw-input RAW-INPUT           Compile <RAW-INPUT> instead of reading <INPUT>
    --dump-hir-to PATH              Dump the hir session to <PATH> as a json file. If <PATH> is `STDOUT`, it dumps the
                                    result to stdout.
    --dump-mir-to PATH              Dump the mir session to <PATH> as a json file. If <PATH> is `STDOUT`, it dumps the
                                    result to stdout.
    --dump-type [json|string]       Set the type of the hir/mir dump (default: json)
    --verbose [0|1|2]               Set verbosity (default: 1)
                                    Set it to 0 to silence it. Set it to 2 for verbose output.
    --or-pattern-limit LIMIT        `|` operators in patterns are implemented very naively: the compiler
                                    just expands all the patterns. This can lead to exponential expansion, so
                                    there's a hard limit for the usage of `|`s. You can set the limit with this
                                    option. (default: 1024)
";

pub const MAJOR_VERSION: u8 = 0;
pub const MINOR_VERSION: u8 = 0;
pub const PATCH_VERSION: u8 = 0;
