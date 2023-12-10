use crate::{
    COMPILER_HELP_MESSAGE,
    MAJOR_VERSION,
    MINOR_VERSION,
    PATCH_VERSION,
};

pub struct CompilerOption {
    pub do_not_compile_and_print_this: Option<String>,
    input_path: String,
    output_path: String,
    format_from: IrPass,
    format_to: IrPass,
    show_warnings: bool,
    save_ir: bool,
    dump_hir: bool,
}

impl CompilerOption {
    pub fn help_message() -> Self {
        CompilerOption::print_this_and_quit(COMPILER_HELP_MESSAGE.to_string())
    }

    pub fn version_message() -> Self {
        CompilerOption::print_this_and_quit(format!("Version {MAJOR_VERSION}.{MINOR_VERSION}.{PATCH_VERSION}"))
    }

    pub fn print_this_and_quit(s: String) -> Self {
        CompilerOption {
            do_not_compile_and_print_this: Some(s),
            ..CompilerOption::default()
        }
    }
}

impl Default for CompilerOption {
    fn default() -> Self {
        CompilerOption {
            do_not_compile_and_print_this: None,
            input_path: String::new(),
            output_path: String::from("./a.out"),
            format_from: IrPass::Code,

            // TODO: it has to be IrPass::Binary, but that's not implemented yet
            format_to: IrPass::HighIr,
            show_warnings: true,
            save_ir: true,
            dump_hir: false,
        }
    }
}

pub enum IrPass {
    Code, Tokens, HighIr,
}

pub fn parse_args() -> Result<CompilerOption, String> {
    let args = std::env::args().collect::<Vec<String>>();

    if args.len() == 1 {
        Err(String::from("no input file\nTry `sodigy --help`"))
    }

    else if args.len() == 2 {
        if let Some(flag) = parse_flag(&args[1]) {
            match flag {
                CompilerFlag::Help => {
                    Ok(CompilerOption::help_message())
                },
                CompilerFlag::Version => {
                    Ok(CompilerOption::version_message())
                },
                CompilerFlag::Output
                | CompilerFlag::From
                | CompilerFlag::To
                | CompilerFlag::ShowWarnings
                | CompilerFlag::SaveIr
                | CompilerFlag::DumpHir => todo!(),
            }
        }

        else {
            Ok(CompilerOption {
                input_path: args[1].clone(),
                ..CompilerOption::default()
            })
        }
    }

    else {
        let mut index = 1;

        todo!()
    }
}

enum CompilerFlag {
    Output,
    From,
    To,
    ShowWarnings,
    SaveIr,
    DumpHir,
    Help,
    Version,
}

fn parse_flag(s: &str) -> Option<CompilerFlag> {
    if s == "-o" || s == "--output" {
        Some(CompilerFlag::Output)
    }

    else if s == "-f" || s == "--from" {
        Some(CompilerFlag::From)
    }

    else if s == "-t" || s == "--to" {
        Some(CompilerFlag::To)
    }

    else if s == "-h" || s == "--help" {
        Some(CompilerFlag::Help)
    }

    else if s == "-v" || s == "--version" {
        Some(CompilerFlag::Version)
    }

    else if s == "--show-warnings" {
        Some(CompilerFlag::ShowWarnings)
    }

    else if s == "--save-ir" {
        Some(CompilerFlag::SaveIr)
    }

    else if s == "--dump-hir" {
        Some(CompilerFlag::DumpHir)
    }

    else {
        None
    }
}
