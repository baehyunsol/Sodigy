use crate::Session;
use sodigy_error::{Error, ErrorKind};
use sodigy_name_analysis::IdentWithOrigin;
use sodigy_parse::{self as ast, Field};
use sodigy_span::Span;
use sodigy_string::InternedString;

#[derive(Clone, Debug)]
pub struct Path {
    pub id: IdentWithOrigin,
    pub fields: Vec<Field>,
}

impl Path {
    pub fn from_ast(ast_path: &ast::Path, session: &mut Session) -> Result<Path, ()> {
        let id = match session.find_origin_and_count_usage(ast_path.id) {
            Some((origin, def_span)) => IdentWithOrigin {
                id: ast_path.id,
                span: ast_path.id_span,
                origin,
                def_span,
            },
            None => {
                session.errors.push(Error {
                    kind: ErrorKind::UndefinedName(ast_path.id),
                    spans: ast_path.id_span.simple_error(),
                    note: None,
                });
                return Err(());
            },
        };

        Ok(Path {
            id,
            fields: ast_path.fields.clone(),
        })
    }

    pub fn error_span_narrow(&self) -> Span {
        match self.fields.get(0) {
            Some(Field::Name { dot_span, .. }) => *dot_span,
            _ => self.id.span,
        }
    }

    pub fn error_span_wide(&self) -> Span {
        let mut span = self.id.span;

        for field in self.fields.iter() {
            match field {
                Field::Name { dot_span, name_span, .. } => {
                    span = span.merge(*dot_span);
                    span = span.merge(*name_span);
                },
                _ => todo!(),
            }
        }

        span
    }

    pub fn unintern_or_default(&self, intermediate_dir: &str) -> String {
        let mut result = self.id.id.unintern_or_default(intermediate_dir);

        for field in self.fields.iter() {
            match field {
                Field::Name { name, .. } => {
                    result = format!("{result}.{}", name.unintern_or_default(intermediate_dir));
                },
                _ => todo!(),
            }
        }

        result
    }
}

impl Path {
    pub fn replace_name_and_span(&mut self, name: InternedString, span: Span) {
        self.id.id = name;
        self.id.span = span;
        self.fields = self.fields.iter().map(
            |field| match field {
                Field::Name { name, .. } => Field::Name {
                    name: *name,
                    name_span: span,
                    dot_span: span,
                    is_from_alias: true,
                },
                _ => unreachable!(),
            }
        ).collect();
    }
}
