use crate::{
    Attribute,
    AttributeKind,
    AttributeRule,
    Expr,
    Requirement,
    Session,
    Type,
    Visibility,
};
use sodigy_name_analysis::{NameOrigin, Namespace};
use sodigy_parse as ast;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Let {
    pub visibility: Visibility,
    pub keyword_span: Span,
    pub name: InternedString,
    pub name_span: Span,
    pub type_annot: Option<Type>,
    pub value: Expr,
    pub origin: LetOrigin,

    // We have to do cycle checks.
    pub foreign_names: HashMap<InternedString, (NameOrigin, Span /* def_span */)>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LetOrigin {
    TopLevel,
    Inline,  // `let` keyword in an inline block
    FuncDefaultValue,
}

impl Let {
    pub fn from_ast(
        ast_let: &ast::Let,
        session: &mut Session,
        is_top_level: bool,
    ) -> Result<Let, ()> {
        let mut has_error = false;
        let mut type_annot = None;

        let attribute = match session.lower_attribute(
            &ast_let.attribute,
            AttributeKind::Let,
            ast_let.keyword_span,
            is_top_level,
        ) {
            Ok(attribute) => attribute,
            Err(()) => {
                has_error = true;
                Attribute::new()
            },
        };
        let visibility = attribute.visibility.clone();

        if let Some(ast_type) = &ast_let.type_annot {
            match Type::from_ast(ast_type, session) {
                Ok(ty) => {
                    type_annot = Some(ty);
                },
                Err(()) => {
                    has_error = true;
                },
            }
        }

        session.name_stack.push(Namespace::ForeignNameCollector {
            is_func: false,
            foreign_names: HashMap::new(),
        });

        let value = match Expr::from_ast(&ast_let.value, session) {
            Ok(value) => Some(value),
            Err(()) => {
                has_error = true;
                None
            },
        };

        let Some(Namespace::ForeignNameCollector { foreign_names, .. }) = session.name_stack.pop() else { unreachable!() };

        if has_error {
            Err(())
        }

        else {
            Ok(Let {
                visibility,
                keyword_span: ast_let.keyword_span,
                name: ast_let.name,
                name_span: ast_let.name_span,
                type_annot,
                value: value.unwrap(),
                origin: if is_top_level {
                    LetOrigin::TopLevel
                } else {
                    LetOrigin::Inline
                },
                foreign_names,
            })
        }
    }

    pub fn get_attribute_rule(is_top_level: bool, _is_std: bool, _session: &Session) -> AttributeRule {
        AttributeRule {
            doc_comment: if is_top_level { Requirement::Maybe } else { Requirement::Never },
            doc_comment_error_note: Some(String::from("You can only add doc comments to top-level items.")),
            visibility: if is_top_level { Requirement::Maybe } else { Requirement::Never },
            visibility_error_note: Some(String::from("Only top-level items can be public.")),
            decorators: HashMap::new(),
        }
    }
}
