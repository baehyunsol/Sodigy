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
            Flag::DumpHir => "dump-hir",
        }.to_string()
    }
}
