#![deny(unused_imports)]

use sodigy::run;
use sodigy_clap::parse_cli_args;
use sodigy_error::SodigyError;

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
        let mut errors_and_warnings = run(clap_result.result);

        errors_and_warnings.print_results();
    }
}
