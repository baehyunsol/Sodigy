use super::RenderSpanSession;
use crate::{RenderableSpan, Span};
use sodigy_endec::Endec;
use sodigy_fs_api::{FileError, FileErrorKind, join4, read_bytes};
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct MonomorphizationInfo {
    pub id: u128,

    // There might be monomorphizations inside another monomorphization.
    pub parent: Option<u128>,

    // "eq<Char>", "add<Int, Int, Int>"
    pub info: String,

    // call span of "add<Int, Int, Int>()"
    // There can be multiple call spans, but for error messages, 1 call span is enough.
    pub span: Span,
}

impl RenderSpanSession {
    // FIXME: There are too many unwraps.
    //        In order for it to return `Result<_, _>`, a lot of APIs have to be changed.
    //        So it just assumes that the compiler correctly stored the data in the intermediate_dir.
    pub fn get_monomorphization_info(&mut self, monomorphization_id: u128) -> Result<MonomorphizationInfo, FileError> {
        let id_str = format!("{monomorphization_id:x}");
        let mono_info_at = join4(
            &self.intermediate_dir,
            "mono",
            id_str.get(0..2).unwrap(),
            id_str.get(2..).unwrap(),
        )?;
        let mono_info_bytes = read_bytes(&mono_info_at)?;
        let mono_info = MonomorphizationInfo::decode(&mono_info_bytes).map_err(|_| FileError {
            kind: FileErrorKind::CannotDecodeFile,
            given_path: Some(mono_info_at.to_string()),
        })?;
        self.monomorphizations.insert(mono_info.id, mono_info.clone());
        Ok(mono_info)
    }

    pub fn explain_monomorphizations(&mut self, monomorphizations: &HashSet<u128>) -> Vec<RenderableSpan> {
        let mut result = vec![];

        for mono_id in monomorphizations.iter() {
            // TODO: I don't want to unwrap this... but I have to change so many APIs to propagate this error.
            let mono_info = self.get_monomorphization_info(*mono_id).unwrap();

            // TODO: follow parent
            result.push(RenderableSpan {
                span: mono_info.span,
                auxiliary: true,
                note: Some(format!("This is `{}`.", mono_info.info)),
            });
        }

        result
    }
}
