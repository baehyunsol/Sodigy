use crate::{
    Expr,
    Func,
    FuncOrigin,
    Let,
    Session,
    Struct,
};
use sodigy_name_analysis::{Namespace, NamespaceKind};
use sodigy_number::InternedNumber;
use sodigy_parse as ast;
use sodigy_span::Span;

#[derive(Clone, Debug)]
pub struct Block {
    pub group_span: Span,
    pub lets: Vec<Let>,
    pub value: Box<Expr>,
}

impl Block {
    pub fn from_ast(
        ast_block: &ast::Block,
        session: &mut Session,
        top_level: bool,
    ) -> Result<Block, ()> {
        let mut lets = vec![];

        // It's just a dummy value. No one's gonna use this.
        let mut value = Expr::Number {
            n: InternedNumber::zero(),
            span: Span::None,
        };

        let mut has_error = false;

        session.name_stack.push(Namespace::new(NamespaceKind::Block, ast_block.iter_names().map(|(k, v1, v2)| (k, (v1, v2))).collect()));

        for r#let in ast_block.lets.iter() {
            match Let::from_ast(r#let, session, top_level) {
                Ok(l) => {
                    lets.push(l);
                },
                Err(()) => {
                    has_error = true;
                },
            }
        }

        let func_origin = if top_level {
            FuncOrigin::TopLevel
        } else {
            FuncOrigin::Inline
        };

        // All the function declarations are stored in the top-level block.
        for func in ast_block.funcs.iter() {
            match Func::from_ast(func, session, func_origin) {
                Ok(f) => {
                    session.funcs.push(f);
                },
                Err(()) => {
                    has_error = true;
                },
            }
        }

        // All the struct declarations are stored in the top-level block.
        for r#struct in ast_block.structs.iter() {
            match Struct::from_ast(r#struct, session) {
                Ok(s) => {
                    session.structs.push(s);
                },
                Err(()) => {
                    has_error = true;
                },
            }
        }

        // If `ast_block.value` is None, that means the block is top-level.
        // An ast_block can be top-level or inline, but an hir_block is always an inline block.
        // If it's a top-level block, `HirSession::lower` will do proper handlings, so this function doesn't have to worry about anything.
        if let Some(ast_value) = ast_block.value.as_ref() {
            match Expr::from_ast(ast_value, session) {
                Ok(v) => {
                    value = v;
                },
                Err(()) => {
                    has_error = true;
                },
            }
        }

        session.name_stack.pop();

        if has_error {
            Err(())
        }

        else {
            Ok(Block {
                group_span: ast_block.group_span,
                lets,
                value: Box::new(value),
            })
        }
    }
}
