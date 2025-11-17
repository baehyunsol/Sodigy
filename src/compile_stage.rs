#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompileStage {
    // stage 1: lex -> parse -> hir (high-level intermediate representation)
    // HIR is like AST, but has a little more information.
    // HIR is created per-file, and does not depend on any other files.
    // That means if some files in a project are modified and some are not modified,
    // the unmodified files will generate the exact same HIR. So, it's cached for
    // incremental compilations.
    Lex, Parse, Hir,

    // HIR has a map of definitions and def_spans, per file. In inter-hir stage,
    // it reads HIRs of all files, and creates a giant map of def_spans.
    // Then, it *resolves* the names in HIRs. After that, all the identifiers in the
    // project are mapped to their def_spans.
    InterHir,

    // MIR is like HIR, but has some extra information for type-checking.
    // MIR is created per-file, but it needs the map in the inter-hir, so
    // you can't cache MIRs.
    Mir,
    TypeCheck,
    Bytecode,
    CodeGen,
}

pub const COMPILE_STAGES: [CompileStage; 8] = [
    CompileStage::Lex,
    CompileStage::Parse,
    CompileStage::Hir,
    CompileStage::InterHir,
    CompileStage::Mir,
    CompileStage::TypeCheck,
    CompileStage::Bytecode,
    CompileStage::CodeGen,
];
