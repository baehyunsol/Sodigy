use crate::Session;
use sodigy_hir::{
    Pattern,
    PatternKind,
};

impl Session {
    pub fn resolve_pattern(&mut self, pattern: &mut Pattern) -> Result<(), ()> {
        let mut has_error = false;

        if let Err(()) = self.resolve_pattern_kind(&mut pattern.kind) {
            has_error = true;
        }

        if has_error {
            Err(())
        }

        else {
            Ok(())
        }
    }

    pub fn resolve_pattern_kind(&mut self, kind: &mut PatternKind) -> Result<(), ()> {
        match kind {
            PatternKind::Path(path) => self.resolve_path(path, None, &mut vec![]),
            PatternKind::Constant(_) |
            PatternKind::NameBinding { .. } |
            PatternKind::Regex { .. } |
            PatternKind::Wildcard(_) => Ok(()),
            PatternKind::Struct { r#struct, fields, .. } => {
                let mut has_error = self.resolve_path(r#struct, None, &mut vec![]).is_err();

                for field in fields.iter_mut() {
                    if let Err(()) = self.resolve_pattern(&mut field.pattern) {
                        has_error = true;
                    }
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(())
                }
            },
            PatternKind::TupleStruct { r#struct, elements, .. } => {
                let mut has_error = self.resolve_path(r#struct, None, &mut vec![]).is_err();

                for element in elements.iter_mut() {
                    if let Err(()) = self.resolve_pattern(element) {
                        has_error = true;
                    }
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(())
                }
            },
            PatternKind::Tuple { elements, .. } |
            PatternKind::List { elements, .. } => {
                let mut has_error = false;

                for element in elements.iter_mut() {
                    if let Err(()) = self.resolve_pattern(element) {
                        has_error = true;
                    }
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(())
                }
            },
            PatternKind::Range { lhs, rhs, .. } => {
                let mut has_error = false;

                if let Some(lhs) = lhs {
                    if let Err(()) = self.resolve_pattern(lhs) {
                        has_error = true;
                    }
                }

                if let Some(rhs) = rhs {
                    if let Err(()) = self.resolve_pattern(rhs) {
                        has_error = true;
                    }
                }

                if has_error {
                    Err(())
                }

                else {
                    Ok(())
                }
            },
            PatternKind::Or { lhs, rhs, .. } => match (
                self.resolve_pattern(lhs),
                self.resolve_pattern(rhs),
            ) {
                (Ok(()), Ok(())) => Ok(()),
                _ => Err(()),
            },
        }
    }
}
