#![deny(unused_imports)]

use sodigy::{
    COMPILER_HELP_MESSAGE,
    MAJOR_VERSION,
    MINOR_VERSION,
    PATCH_VERSION,
    result::ErrorsAndWarnings,
    stages::{hir_stage, parse_stage},
};
use sodigy_clap::{parse_cli_args, IrStage, SpecialOutput};
use sodigy_error::SodigyError;
use sodigy_files::global_file_session;

fn main() {
    // test purpose
    std::env::set_var("RUST_BACKTRACE", "FULL");

    let clap_result = parse_cli_args();

    for warning in clap_result.warnings {
        println!("{}\n", warning.render_error());
    }

    if !clap_result.errors.is_empty() {
        for error in clap_result.errors.iter() {
            println!("{}\n", error.render_error());
        }

        return;
    }

    else {
        let opt = clap_result.result;

        if let Some(sp) = opt.do_not_compile_and_print_this {
            match sp {
                SpecialOutput::HelpMessage => {
                    println!("{COMPILER_HELP_MESSAGE}");
                },
                SpecialOutput::VersionInfo => {
                    println!("sodigy {MAJOR_VERSION}.{MINOR_VERSION}.{PATCH_VERSION}");
                },
            }

            return;
        }

        let file_session = unsafe { global_file_session() };
        let mut errors_and_warnings = ErrorsAndWarnings::new();
        let output_format = opt.output_format;

        for file in opt.input_files.iter() {
            let file = match file_session.register_file(file) {
                Ok(f) => f,
                Err(e) => {
                    errors_and_warnings.push_error(e.into());
                    continue;
                },
            };

            let save_tokens_to = if output_format == IrStage::Tokens {
                // TODO: what if there are multiple inputs?
                Some(opt.output_path.clone())
            } else if opt.save_ir {
                // TODO: make output_path
                todo!()
            } else {
                None
            };

            let (parse_session, errors_and_warnings_) = parse_stage(file, Some(errors_and_warnings), save_tokens_to);
            errors_and_warnings = errors_and_warnings_;

            if errors_and_warnings.has_error() || output_format == IrStage::Tokens {
                continue;
            }

            let parse_session = if let Some(parse_session) = parse_session {
                parse_session
            } else {
                continue;  // error occured in `parse_stage`
            };

            let save_hir_to = if output_format == IrStage::HighIr {
                // TODO: what if there are multiple inputs?
                Some(opt.output_path.clone())
            } else if opt.save_ir {
                // TODO: make output_path
                todo!()
            } else {
                None
            };

            let (hir_session, errors_and_warnings_) = hir_stage(&parse_session, Some(errors_and_warnings), save_hir_to);
            errors_and_warnings = errors_and_warnings_;

            if errors_and_warnings.has_error() {
                continue;
            }

            let hir_session = if let Some(hir_session) = hir_session {
                hir_session
            } else {
                continue;  // error occured in `parse_stage`
            };

            if opt.dump_hir {
                println!("{}", hir_session.dump_hir());
            }
        }

        errors_and_warnings.print_results();
    }
}
