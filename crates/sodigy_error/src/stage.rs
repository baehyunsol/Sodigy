mod endec;

#[derive(Clone, Copy)]
pub enum Stage {
    FileIo,
    Endec,
    Clap,
    Lex,
    Parse,
    Ast,
    Hir,
    Mir,
}
