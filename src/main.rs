#![deny(unused_imports)]

use sodigy::run;
use sodigy::result::CompilerOutput;
use sodigy_clap::parse_cli_args;
use sodigy_error::SodigyError;

fn main() {
    // test purpose
    std::env::set_var("RUST_BACKTRACE", "FULL");

    let clap_result = parse_cli_args();
    let mut compiler_output = CompilerOutput::new();

    for warning in clap_result.warnings.iter() {
        compiler_output.push_warning(warning.to_universal());
    }

    for error in clap_result.errors.iter() {
        compiler_output.push_error(error.to_universal());
    }

    if clap_result.has_error() {
        println!("{}", compiler_output.concat_results());
    }

    else {
        let mut compiler_output = run(clap_result.result, Some(compiler_output));
        println!("{}", compiler_output.concat_results());
    }
}
