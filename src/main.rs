#![deny(unused_imports)]

use sodigy::run;
use sodigy_clap::parse_cli_args;
use sodigy_error::SodigyError;

fn main() {
    // test purpose
    std::env::set_var("RUST_BACKTRACE", "FULL");

    let clap_result = parse_cli_args();

    for warning in clap_result.warnings {
        println!("{}\n", warning.render_error(true));
    }

    if !clap_result.errors.is_empty() {
        for error in clap_result.errors.iter() {
            println!("{}\n", error.render_error(true));
        }

        return;
    }

    else {
        let mut compiler_output = run(clap_result.result);
        println!("{}", compiler_output.concat_results());
    }
}
