#![deny(unused_imports)]

use sodigy_session::SessionOutput;
use std::collections::HashMap;

pub const MIN_VERBOSITY: u8 = 0;
pub const MAX_VERBOSITY: u8 = 2;

type Path = String;

#[derive(Clone)]
pub struct CompilerOption {
    pub do_not_compile_and_do_this: Option<SpecialOutput>,
    pub input_file: Option<Path>,

    pub output: CompilerOutputFormat,
    pub show_warnings: bool,
    pub dump_hir_to: Option<Path>,
    pub dump_mir_to: Option<Path>,

    // TODO: this doesn't do anything
    pub verbosity: u8,

    // It has to be `Vec<u8>` because the test code has to run
    // invalid utf-8 strings.
    pub raw_input: Option<Vec<u8>>,

    // users cannot set this flag manually
    pub parse_config_file: bool,
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
            input_file: None,
            output: CompilerOutputFormat::None,
            save_ir: false,
            raw_input: Some(code.to_vec()),
            ..Self::default()
        }
    }
}

impl Default for CompilerOption {
    fn default() -> Self {
        CompilerOption {
            do_not_compile_and_do_this: None,
            input_file: None,
            output: CompilerOutputFormat::None,
            show_warnings: true,
            save_ir: true,
            dump_hir_to: None,
            dump_mir_to: None,
            dependencies: HashMap::new(),
            verbosity: 1,
            raw_input: None,
            parse_config_file: false,
            num_workers: calc_num_workers(),
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
    None,
    Path(Path),
    HighIr,
    MidIr,
}

impl CompilerOutputFormat {
    pub fn try_unwrap_path(&self) -> Option<&Path> {
        if let CompilerOutputFormat::Path(p) = self {
            Some(p)
        }

        else {
            None
        }
    }
}

// don't call these. just use session.get_results_mut()
impl SessionOutput<CompilerOption> for CompilerOption {
    fn pop(&mut self) -> Option<CompilerOption> {
        unreachable!()
    }

    fn push(&mut self, _: CompilerOption) {
        unreachable!()
    }

    fn clear(&mut self) {
        unreachable!()
    }

    fn len(&self) -> usize {
        unreachable!()
    }
}
