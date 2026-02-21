mod cli;
mod compile_stage;

pub use cli::{CliCommand, ColorWhen, parse_args};
pub use compile_stage::{CompileStage, COMPILE_STAGES};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Profile {
    Script,
    Test,
}
