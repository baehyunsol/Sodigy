#![deny(unused_imports)]

use std::collections::HashMap;

pub const MIN_VERBOSITY: u8 = 0;
pub const MAX_VERBOSITY: u8 = 2;

type Path = String;

#[derive(Clone)]
pub struct CompilerOption {
    pub do_not_compile_and_do_this: Option<SpecialOutput>,
    pub input_path: Option<Path>,
    pub output_path: Option<Path>,
    pub output_format: CompilerOutputFormat,

    pub show_warnings: bool,
    pub dump_hir_to: Option<Path>,
    pub dump_mir_to: Option<Path>,
    pub dump_type: DumpType,
    pub library_paths: Option<HashMap<String, Path>>,

    // TODO: this doesn't do anything
    pub verbosity: u8,
    pub or_pattern_expansion_limit: usize,

    // It has to be `Vec<u8>` because the test code has to run
    // invalid utf-8 strings.
    pub raw_input: Option<Vec<u8>>,
}

impl CompilerOption {
    pub fn help_message() -> Self {
        CompilerOption::do_this_and_quit(SpecialOutput::HelpMessage)
    }

    pub fn version_info() -> Self {
        CompilerOption::do_this_and_quit(SpecialOutput::VersionInfo)
    }

    pub fn do_this_and_quit(s: SpecialOutput) -> Self {
        CompilerOption {
            do_not_compile_and_do_this: Some(s),
            ..CompilerOption::default()
        }
    }

    pub fn test_runner(code: &[u8]) -> Self {
        CompilerOption {
            do_not_compile_and_do_this: None,
            input_path: None,
            output_path: None,
            raw_input: Some(code.to_vec()),
            ..Self::default()
        }
    }
}

impl Default for CompilerOption {
    fn default() -> Self {
        CompilerOption {
            do_not_compile_and_do_this: None,
            input_path: None,
            output_path: None,
            output_format: CompilerOutputFormat::Binary,
            show_warnings: true,
            dump_hir_to: None,
            dump_mir_to: None,
            dump_type: DumpType::Json,
            library_paths: None,
            verbosity: 1,
            or_pattern_expansion_limit: 1024,
            raw_input: None,
        }
    }
}

#[derive(Clone)]
pub enum SpecialOutput {
    HelpMessage,
    VersionInfo,
}

#[derive(Clone)]
pub enum CompilerOutputFormat {
    Hir,
    Mir,
    Binary,
}

impl CompilerOutputFormat {
    pub fn create_output_path(&self) -> String {
        match self {
            CompilerOutputFormat::Hir => String::from("./a.hir"),
            CompilerOutputFormat::Mir => String::from("./a.mir"),
            CompilerOutputFormat::Binary => String::from("./a.out"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum DumpType {
    Json,
    String,
}
