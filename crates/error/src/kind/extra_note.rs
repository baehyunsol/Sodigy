use super::ErrorKind;
use crate::WarningKind;
use sodigy_name_analysis::NameKind;

impl ErrorKind {
    pub fn extra_notes(&self) -> Vec<String> {
        let mut notes = vec![];

        match self {
            WarningKind::UnusedNames { kind, .. } => {
                match kind {
                    NameKind::Let { .. } | NameKind::FuncParam => {
                        notes.push(String::from("If it's intended, use `#[unused_name]` attribute."));
                    },
                    _ => {},
                }
            },
            WarningKind::NoImpureCallInImpureContext { .. } => {
                notes.push(String::from("If it's intended, use `#[unused_effect]` attribute."));
            },
            _ => {},
        }

        notes
    }
}
