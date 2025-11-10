use crate::{Attribute, AttributeRule, Requirement, Session, Visibility};
use sodigy_parse as ast;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

pub struct Module {
    pub visibility: Visibility,
    pub keyword_span: Span,
    pub name: InternedString,
    pub name_span: Span,
}


impl Module {
    pub fn from_ast(ast_module: &ast::Module, session: &mut Session) -> Result<Module, ()> {
        let mut has_error = false;

        // TODO: I want it to be static
        let attribute_rule = AttributeRule {
            // TODO: I want users to be able to add doc comments to modules, but there's no way we can add doc comments to the lib
            doc_comment: Requirement::Never,
            doc_comment_error_note: Some(String::from("You can't add doc comments to a module.")),

            // NOTE: a module definition is always at top-level
            visibility: Requirement::Maybe,
            visibility_error_note: None,

            decorators: HashMap::new(),
        };

        let attribute = match Attribute::from_ast(&ast_module.attribute, session, &attribute_rule, ast_module.keyword_span) {
            Ok(attribute) => attribute,
            Err(()) => {
                has_error = true;
                Attribute::new()
            },
        };
        let visibility = attribute.visibility;

        if has_error {
            Err(())
        }

        else {
            Ok(Module {
                visibility,
                keyword_span: ast_module.keyword_span,
                name: ast_module.name,
                name_span: ast_module.name_span,
            })
        }
    }
}
