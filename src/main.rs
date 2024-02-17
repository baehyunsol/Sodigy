#![deny(unused_imports)]

use log::{debug, info};
use sodigy::run;
use sodigy::result::CompilerOutput;
use sodigy_clap::parse_cli_args;
use sodigy_error::SodigyError;
use sodigy_session::SodigySession;

const EXIT_SUCCESS: i32 = 0;
const EXIT_FAILURE: i32 = 1;

fn main() {
    env_logger::init();
    info!("env_logger successfully initialized");

    // hack: `RUST_BACKTRACE` is set according to `RUST_LOG`
    info!("std::env::set_var{:?}: RUST_BACKTRACE = 1", std::env::set_var("RUST_BACKTRACE", "1"));
    debug!("std::env::set_var{:?}: RUST_BACKTRACE = FULL", std::env::set_var("RUST_BACKTRACE", "FULL"));

    let clap_result = parse_cli_args();
    let mut has_error = false;
    let mut compiler_output = CompilerOutput::new();

    for warning in clap_result.get_warnings().iter() {
        compiler_output.show_overall_result = true;
        compiler_output.push_warning(warning.to_universal());
    }

    for error in clap_result.get_errors().iter() {
        compiler_output.show_overall_result = true;
        compiler_output.push_error(error.to_universal());
    }

    if clap_result.has_error() {
        println!("{}", compiler_output.concat_results());
        has_error = true;
    }

    else {
        let compiler_output_ = run(clap_result.get_results().clone());
        compiler_output.merge(compiler_output_);

        println!("{}", compiler_output.concat_results());

        if compiler_output.has_error() {
            has_error = true;
        }
    }

    if has_error {
        std::process::exit(EXIT_FAILURE);
    }

    else {
        std::process::exit(EXIT_SUCCESS);
    }
}
