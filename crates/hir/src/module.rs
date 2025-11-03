use crate::{Public, Session};
use sodigy_parse as ast;
use sodigy_span::Span;
use sodigy_string::InternedString;

pub struct Module {
    pub public: Public,
    pub keyword_span: Span,
    pub name: InternedString,
    pub name_span: Span,
}


impl Module {
    pub fn from_ast(ast_module: &ast::Module, session: &mut Session) -> Result<Module, ()> {
        let mut has_error = false;
        let public = match Public::from_ast(&ast_module.attribute.public, session) {
            Ok(p) => Some(p),
            Err(()) => {
                has_error = true;
                None
            },
        };

        if has_error {
            Err(())
        }

        else {
            Ok(Module {
                public: public.unwrap(),
                keyword_span: ast_module.keyword_span,
                name: ast_module.name,
                name_span: ast_module.name_span,
            })
        }
    }
}
