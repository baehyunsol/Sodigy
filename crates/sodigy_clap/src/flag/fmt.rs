use super::Flag;
use sodigy_error::RenderError;

impl RenderError for Flag {
    fn render_error(&self) -> String {
        match self {
            Flag::Output => "output",
            Flag::To => "to",
            Flag::Help => "help",
            Flag::Version => "version",
            Flag::ShowWarnings => "show-warnings",
            Flag::SaveIr => "save-ir",
            Flag::DumpTokens => "dump-tokens",
            Flag::DumpTokensTo => "dump-tokens-to",
            Flag::DumpHir => "dump-hir",
            Flag::DumpHirTo => "dump-hir-to",
            Flag::Clean => "clean",
            Flag::Verbose => "verbose",
            Flag::RawInput => "raw-input",
        }.to_string()
    }
}
