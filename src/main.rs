#![deny(unused_imports)]

use colored::*;
use sodigy::option::parse_args;

fn main() {
    // test purpose
    std::env::set_var("RUST_BACKTRACE", "FULL");

    let compiler_option = match parse_args() {
        Ok(args) => args,
        Err(e) => {
            println!("Sodigy: {}: {e}\n", "error".red());
            return;
        },
    };

    if let Some(s) = compiler_option.do_not_compile_and_print_this {
        println!("{s}");
        return;
    }
}
