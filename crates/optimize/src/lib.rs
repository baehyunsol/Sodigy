mod bytecode;
mod mir;

pub use bytecode::optimize_bytecode;
pub use mir::optimize_mir;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OptimizeLevel {
    None,
    Mild,
    Extreme,
}
