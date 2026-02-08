use crate::{
    ArgCount,
    ArgType,
    AssociatedItem,
    AssociatedItemKind,
    Attribute,
    AttributeRule,
    DecoratorRule,
    Expr,
    FuncOrigin,
    Requirement,
    Session,
    Type,
    TypeAssertion,
    Visibility,
    get_decorator_error_notes,
};
use sodigy_error::ItemKind;
use sodigy_name_analysis::{NameKind, NameOrigin, Namespace};
use sodigy_parse as ast;
use sodigy_span::Span;
use sodigy_string::{InternedString, intern_string};
use sodigy_token::Constant;
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

    // TODO: distinguish struct default values and func default values
    FuncDefaultValue,

    // `match` expressions are lowered to blocks
    Match,
}

#[derive(Clone, Debug)]
/// Sessions keep track of simple `let` statements.
/// This information is later used for various optimizations.
pub enum TrivialLet {
    /// `let x = 3;`
    Constant(Constant),

    /// `let x = y;`
    Reference(Span /* def_span of rhs */),

    /// `let x = \() => ...;`
    ///
    /// If the lambda does not capture any name, we can later turn
    /// it into `IsLambda`.
    MaybeLambda(Span /* def_span of the lambda */),

    /// We don't want to evaluate it at runtime.
    /// We don't want the cycle-checker to reject a recursive lambda.
    IsLambda(Span /* def_span of the lambda */),
}

impl Let {
    pub fn from_ast(ast_let: &ast::Let, session: &mut Session) -> Result<Let, ()> {
        let mut has_error = false;
        let mut type_annot = None;

        let attribute = match session.lower_attribute(
            &ast_let.attribute,
            ItemKind::Let,
            ast_let.keyword_span,
        ) {
            Ok(attribute) => attribute,
            Err(()) => {
                has_error = true;
                Attribute::new()
            },
        };
        let visibility = attribute.visibility.clone();

        if let Some(asserted_type) = attribute.get_decorator(b"assert_type", &session.intermediate_dir) {
            session.type_assertions.push(TypeAssertion {
                name_span: ast_let.name_span,
                type_span: asserted_type.args[0].error_span_wide(),
                r#type: asserted_type.args[0].clone().unwrap_type(),
            });
        }

        if let Some(association) = attribute.get_decorator(b"associate", &session.intermediate_dir) {
            session.associated_items.push(AssociatedItem {
                kind: AssociatedItemKind::Let,
                name: ast_let.name,
                name_span: ast_let.name_span,
                is_pure: None,
                params: None,
                type_span: association.args[0].error_span_wide(),
                r#type: association.args[0].clone().unwrap_type(),
            });
        }

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
            Ok(value) => {
                if let Some(t) = session.check_trivial_value(&value) {
                    session.trivial_lets.insert(ast_let.name_span, t);
                }

                Some(value)
            },
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
                origin: if session.is_at_top_level_block() {
                    LetOrigin::TopLevel
                } else {
                    LetOrigin::Inline
                },
                foreign_names,
            })
        }
    }

    pub fn get_attribute_rule(is_top_level: bool, is_std: bool, intermediate_dir: &str) -> AttributeRule {
        let mut attribute_rule = AttributeRule {
            doc_comment: if is_top_level { Requirement::Maybe } else { Requirement::Never },
            doc_comment_error_note: Some(String::from("You can only add doc comments to top-level items.")),
            visibility: if is_top_level { Requirement::Maybe } else { Requirement::Never },
            visibility_error_note: Some(String::from("Only top-level items can be public.")),
            decorators: vec![
                (
                    intern_string(b"assert_type", intermediate_dir).unwrap(),
                    DecoratorRule {
                        name: intern_string(b"assert_type", intermediate_dir).unwrap(),
                        requirement: Requirement::Maybe,
                        arg_requirement: Requirement::Must,
                        arg_count: ArgCount::Eq(1),
                        arg_type: ArgType::Type,
                        arg_type_error_note: Some(String::from("Please give me the type of the value.")),
                        ..DecoratorRule::default()
                    },
                ), (
                    intern_string(b"associate", intermediate_dir).unwrap(),
                    DecoratorRule {
                        name: intern_string(b"associate", intermediate_dir).unwrap(),
                        requirement: Requirement::Maybe,
                        arg_requirement: Requirement::Must,
                        arg_count: ArgCount::Eq(1),
                        arg_count_error_note: Some(String::from("You can associate at most 1 type with a value.")),
                        arg_type: ArgType::Type,
                        arg_type_error_note: Some(String::from("The argument must be a type that you want to associate the value with.")),
                        ..DecoratorRule::default()
                    },
                ),
            ].into_iter().collect(),
            decorator_error_notes: get_decorator_error_notes(ItemKind::Let, intermediate_dir),
        };

        if is_std {
            attribute_rule.add_decorators_for_std(ItemKind::Let, intermediate_dir);
        }

        attribute_rule
    }
}

impl Session {
    pub fn check_trivial_value(&self, value: &Expr) -> Option<TrivialLet> {
        match value {
            Expr::Path(p) if p.fields.is_empty() && p.types[0].is_none() => match p.id.origin {
                NameOrigin::Foreign { kind: NameKind::Func } => {
                    // FIXME: linear search
                    for func in self.funcs.iter() {
                        if func.name_span == p.id.def_span && func.origin == FuncOrigin::Lambda {
                            return Some(TrivialLet::MaybeLambda(p.id.def_span));
                        }
                    }

                    // `let f = \(x) if x == 0 { 0 } else { 1 + f(x - 1) };`
                    // ->
                    // In this case, 1) `f` doesn't have to be evaluated at runtime and
                    // 2) the cycle-checker should not reject `f`.
                    for block in self.block_stack.iter() {
                        for lambda in block.lambdas.iter() {
                            if lambda.name_span == p.id.def_span {
                                return Some(TrivialLet::MaybeLambda(p.id.def_span));
                            }
                        }
                    }

                    Some(TrivialLet::Reference(p.id.def_span))
                },
                _ => Some(TrivialLet::Reference(p.id.def_span)),
            },
            Expr::Constant(c) => Some(TrivialLet::Constant(c.clone())),
            Expr::Block(b) if b.asserts.is_empty() => match self.check_trivial_value(&b.value) {
                // There are too many edge cases...
                // Also, the optimizer can easily optimize this case even if `check_trivial_value` doesn't give a hint.
                Some(TrivialLet::Reference(_)) => None,

                r @ (Some(TrivialLet::Constant(_)) | Some(TrivialLet::MaybeLambda(_)) | Some(TrivialLet::IsLambda(_))) => r,
                None => None,
            },
            _ => None,
        }
    }
}
