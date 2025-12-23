use crate::{
    Attribute,
    AttributeRule,
    Requirement,
    Session,
    Visibility,
    get_decorator_error_notes,
};
use sodigy_error::ItemKind;
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

        let attribute = match session.lower_attribute(
            &ast_module.attribute,
            ItemKind::Module,
            ast_module.keyword_span,
            true,  // a module is always at top level
        ) {
            Ok(attribute) => attribute,
            Err(()) => {
                has_error = true;
                Attribute::new()
            },
        };
        let visibility = attribute.visibility.clone();

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

    pub fn get_attribute_rule(_is_top_level: bool, is_std: bool, intermediate_dir: &str) -> AttributeRule {
        let mut attribute_rule = AttributeRule {
            doc_comment: Requirement::Never,
            doc_comment_error_note: Some(String::from("Use module doc comment inside the module instead.")),
            visibility: Requirement::Maybe,
            visibility_error_note: None,
            decorators: HashMap::new(),
            decorator_error_notes: get_decorator_error_notes(ItemKind::Module, intermediate_dir),
        };

        if is_std {
            attribute_rule.add_decorators_for_std(ItemKind::Module, intermediate_dir);
        }

        attribute_rule
    }
}
