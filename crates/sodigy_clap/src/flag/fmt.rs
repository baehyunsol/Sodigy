use super::Flag;
use sodigy_error::RenderError;

impl RenderError for Flag {
    fn render_error(&self) -> String {
        match self {
            Flag::Output => "output",
            Flag::StopAt => "stop-at",
            Flag::Help => "help",
            Flag::Version => "version",
            Flag::ShowWarnings => "show-warnings",
            Flag::SaveIr => "save-ir",
            Flag::DumpHirTo => "dump-hir-to",
            Flag::DumpMirTo => "dump-mir-to",
            Flag::Clean => "clean",
            Flag::Verbose => "verbose",
            Flag::RawInput => "raw-input",
        }.to_string()
    }
}
