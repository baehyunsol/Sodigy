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

    match parse_cli_args() {
        Ok(opt) => {
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
        Err(e) => {
            for e in e.iter() {
                println!("{}\n", e.render_error());
            }

            return;
        },
    }
}
