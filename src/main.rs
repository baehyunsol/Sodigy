#![deny(unused_imports)]

use sodigy::{
    COMPILER_HELP_MESSAGE,
    MAJOR_VERSION,
    MINOR_VERSION,
    PATCH_VERSION,
};
use sodigy_clap::{parse_cli_args, SpecialOutput};
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

        todo!()
    }
}
