use crate::Session;
use sodigy_error::{Error, ErrorKind};
use sodigy_parse as ast;
use sodigy_span::Span;
use sodigy_string::{
    InternedString,
    intern_string,
    unintern_string,
};

// TODO: more fine-grained publicity
#[derive(Clone, Debug)]
pub struct Public(pub bool);

impl Public {
    // TODO: more fine-grained publicity
    pub fn from_ast(ast_public: &Option<ast::Public>, session: &mut Session) -> Result<Public, ()> {
        Ok(Public(ast_public.is_some()))
    }

    // TODO: more fine-grained publicity
    pub fn private() -> Self {
        Public(false)
    }

    // TODO: more fine-grained publicity
    pub fn is_public(&self) -> bool {
        self.0
    }
}

// There are special decorators for items in Sodigy std.
#[derive(Clone, Debug)]
pub struct StdAttribute {
    pub built_in: bool,
    pub no_type: bool,
    pub lang_item: Option<String>,
    pub lang_item_generics: Vec<String>,
}

impl StdAttribute {
    // TODO: I really hate macros, but I guess we need a macro for parsing attributes...
    //       `AssertAttribute::from_ast` and `StdAttribute::from_ast` are almost identical,
    //       and we have to repeat this for `FuncAttribute`, `LetAttribute`, `StructAttribute`, ...
    //       ----
    //       Current attributes:
    //           `@lang_item("name")`
    //           `@name("name")`: takes exactly 1 input, which must be a string literal
    //
    //           `@note(f"expected {answer}, got {result}")`: takes exactly 1 input, which can be an arbitrary expression
    //
    //           `@lang_item_generic("op.div.generic.0")`: takes 1 or more inputs, which must be string literals
    //
    //           `@always`
    //           `@built_in`
    //           `@no_type`: takes no input
    pub fn from_ast(ast_attribute: &ast::Attribute, session: &mut Session) -> Result<StdAttribute, ()> {
        let mut built_in = false;
        let mut no_type = false;
        let mut lang_item = None;
        let mut lang_item_generics = vec![];

        for decorator in ast_attribute.decorators.iter() {
            todo!()
        }

        Ok(StdAttribute {
            built_in,
            no_type,
            lang_item,
            lang_item_generics,
        })
    }

    pub fn new() -> StdAttribute {
        StdAttribute {
            built_in: false,
            no_type: false,
            lang_item: None,
            lang_item_generics: vec![],
        }
    }
}

impl Session {
    // If you want to set `max_len` to `N`, please make sure that the first `N` elements of `decorator_name` are correct.
    // Cuz the error message will suggest so.
    pub fn error_if_decorator_name_too_long(&mut self, has_error: &mut bool, decorator_name: &[(InternedString, Span)], max_len: usize) {
        if decorator_name.len() > max_len {
            let mut uninterned_names = Vec::with_capacity(decorator_name.len());
            let mut merged_span = decorator_name[0].1;

            for (name, span) in decorator_name.iter() {
                uninterned_names.push(unintern_string(*name, &self.intermediate_dir).unwrap().unwrap());
                merged_span = merged_span.merge(*span);
            }

            let mut merged_name = intern_string(&uninterned_names.clone().join(&(b".")[..]), &self.intermediate_dir).unwrap();
            self.errors.push(Error {
                kind: ErrorKind::InvalidDecorator(merged_name),
                spans: merged_span.simple_error(),
                note: Some(format!(
                    "Perhaps you mean `{}`?",
                    String::from_utf8_lossy(&uninterned_names[..max_len].to_vec().join(&(b".")[..])),
                )),
            });
            *has_error = true;
        }
    }
}
