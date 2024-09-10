use crate::{Delim, IdentWithSpan, Punct, TokenTree, TokenTreeKind};
use crate::error::{ParseError, ParseErrorKind};
use crate::warn::{ParseWarning, ParseWarningKind};
use smallvec::smallvec;
use sodigy_config::CompilerOption;
use sodigy_error::{ErrorContext, ExpectedToken, SodigyError, UniversalError};
use sodigy_intern::{InternedString, InternSession};
use sodigy_lex::LexSession;
use sodigy_session::{
    SessionSnapshot,
    SodigySession,
};
use sodigy_span::SpanRange;
use std::collections::HashMap;

mod endec;

pub struct ParseSession {
    tokens: Vec<TokenTree>,
    errors: Vec<ParseError>,
    warnings: Vec<ParseWarning>,
    interner: InternSession,
    snapshots: Vec<SessionSnapshot>,
    compiler_option: CompilerOption,

    // errors from `LexSession`
    previous_errors: Vec<UniversalError>,
    previous_warnings: Vec<UniversalError>,

    // names of unexpanded macros
    // parse_session will look for the definitions of these macros later
    // its span points to its name and the square brackets
    pub unexpanded_macros: HashMap<InternedString, SpanRange>,
}

impl ParseSession {
    pub fn from_lex_session(session: &LexSession) -> Self {
        ParseSession {
            tokens: vec![],
            errors: vec![],
            warnings: vec![],
            interner: session.get_interner_cloned(),
            snapshots: vec![],
            unexpanded_macros: HashMap::new(),
            compiler_option: session.get_compiler_option().clone(),
            previous_errors: session.get_all_errors(),
            previous_warnings: session.get_all_warnings(),
        }
    }

    /// EXPENSIVE
    pub fn dump_tokens(&self) -> String {
        self.tokens.iter().map(|t| t.to_string()).collect::<Vec<String>>().join(" ")
    }

    pub fn push_field_modifier(&mut self, id: InternedString, span: SpanRange) {
        match self.tokens.last_mut() {
            Some(TokenTree {
                kind: TokenTreeKind::Punct(Punct::FieldModifier(fields)),
                span: prev_span,
            }) => {
                fields.push(IdentWithSpan::new(id, span));
                *prev_span = prev_span.merge(span);
            },
            _ => {
                self.tokens.push(TokenTree {
                    kind: TokenTreeKind::Punct(Punct::FieldModifier(smallvec![IdentWithSpan::new(id, span)])),
                    span,
                });
            },
        }
    }

    // it finds `@[...]`s and replace them with `TokenTree::Macro`
    // it tries to expand the macro if it can
    pub fn replace_macro_tokens(&mut self) -> Result<(), ()> {
        let mut new_tokens = Vec::with_capacity(self.tokens.len());
        let mut errors = vec![];
        let mut curr_state = ExpandState::Init;

        let mut curr_macro_span = self.tokens.get(0).map(|token| token.span).unwrap_or(SpanRange::dummy());
        let mut curr_macro_name_tokens = vec![];
        let mut curr_macro_args;

        for token in self.tokens.iter() {
            match curr_state {
                ExpandState::Init => {
                    if token.kind == TokenTreeKind::Punct(Punct::At) {
                        curr_state = ExpandState::TryReadMacroName;
                    }

                    new_tokens.push(token.clone());
                },
                ExpandState::TryReadMacroName => {
                    let curr_span = token.span;

                    if let TokenTreeKind::Group {
                        delim: Delim::Bracket,
                        prefix: b'\0',
                        tokens,
                    } = &token.kind {
                        // `@` it just pushed is a macro!
                        let at = new_tokens.pop().unwrap();

                        curr_macro_span = at.span.merge(curr_span);
                        curr_macro_name_tokens = tokens.clone();
                        curr_state = ExpandState::ReadMacroArgs;
                    }

                    else {
                        curr_state = ExpandState::Init;
                        new_tokens.push(token.clone());
                    }
                },
                ExpandState::ReadMacroArgs => {
                    let curr_span = token.span;

                    if let TokenTreeKind::Group {
                        delim: Delim::Paren,
                        prefix: b'\0',
                        tokens,
                    } = &token.kind {
                        curr_macro_args = tokens.clone();
                        let parent_span = curr_macro_span;
                        curr_macro_span = curr_macro_span.merge(curr_span);

                        match try_unwrap_macro_name(&curr_macro_name_tokens, parent_span) {
                            Ok((name, span)) => {
                                if let Some(tokens) = self.try_expand_macro(
                                    name,
                                    &curr_macro_args,
                                ) {
                                    for token in tokens.into_iter() {
                                        new_tokens.push(token);
                                    }
                                }

                                else {
                                    new_tokens.push(TokenTree {
                                        kind: TokenTreeKind::Macro {
                                            name,
                                            args: curr_macro_args.clone(),
                                        },
                                        span,
                                    });
                                    self.unexpanded_macros.insert(name, parent_span);
                                }
                            },
                            Err(e) => {
                                errors.push(e);
                            },
                        }

                        curr_macro_name_tokens.clear();
                        curr_macro_args.clear();
                    }

                    else {
                        errors.push(
                            ParseError::unexpected_token(
                                token.clone(),
                                ExpectedToken::Specific(vec![TokenTreeKind::Group {
                                    delim: Delim::Paren,
                                    prefix: b'\0',
                                    tokens: vec![],
                                }])
                            ).set_error_context(
                                ErrorContext::ExpandingMacro
                            ).to_owned()
                        );
                    }

                    curr_state = ExpandState::Init;
                },
            }
        }

        for error in errors.into_iter() {
            self.push_error(error);
        }

        self.tokens = new_tokens;
        self.err_if_has_error()?;

        Ok(())
    }

    // `()` in `macro_definitions: HashMap<InternedString, ()>` means `TODO`
    pub fn expand_macros(&mut self, macro_definitions: &HashMap<InternedString, ()>) -> Result<(), ()> {
        // TODO
        // 1. iterate all the tokens and find `TokenTree::Macro`s.
        // 2. when macros are found, try to expand that with `macro_definitions`
        // 3. return Err(()) if it fails

        Ok(())
    }

    // it expands compiler-builtin macros
    // for now, there are none
    pub fn try_expand_macro(
        &self,
        name: InternedString,
        args: &[TokenTree],
    ) -> Option<Vec<TokenTree>> {
        None
    }
}

fn try_unwrap_macro_name(tokens: &[TokenTree], parent_span: SpanRange) -> Result<(InternedString, SpanRange), ParseError> {
    match tokens.len() {
        1 => match &tokens[0].kind {
            TokenTreeKind::Identifier(id) => Ok((*id, tokens[0].span)),
            _ => Err(ParseError::unexpected_token(
                tokens[0].clone(),
                ExpectedToken::ident(),
            )),
        },
        0 => Err(ParseError::unexpected_eof(
            ExpectedToken::ident(),
            parent_span,
        )),
        _ => Err(ParseError::unexpected_token(
            tokens[1].clone(),
            ExpectedToken::nothing(),
        )),
    }
}

impl SodigySession<ParseError, ParseErrorKind, ParseWarning, ParseWarningKind, Vec<TokenTree>, TokenTree> for ParseSession {
    fn get_errors(&self) -> &Vec<ParseError> {
        &self.errors
    }

    fn get_errors_mut(&mut self) -> &mut Vec<ParseError> {
        &mut self.errors
    }

    fn get_warnings(&self) -> &Vec<ParseWarning> {
        &self.warnings
    }

    fn get_warnings_mut(&mut self) -> &mut Vec<ParseWarning> {
        &mut self.warnings
    }

    fn get_previous_errors(&self) -> &Vec<UniversalError> {
        &self.previous_errors
    }

    fn get_previous_errors_mut(&mut self) -> &mut Vec<UniversalError> {
        &mut self.previous_errors
    }

    fn get_previous_warnings(&self) -> &Vec<UniversalError> {
        &self.previous_warnings
    }

    fn get_previous_warnings_mut(&mut self) -> &mut Vec<UniversalError> {
        &mut self.previous_warnings
    }

    fn get_results(&self) -> &Vec<TokenTree> {
        &self.tokens
    }

    fn get_results_mut(&mut self) -> &mut Vec<TokenTree> {
        &mut self.tokens
    }

    fn get_interner(&mut self) -> &mut InternSession {
        &mut self.interner
    }

    fn get_interner_cloned(&self) -> InternSession {
        self.interner.clone()
    }

    fn get_snapshots_mut(&mut self) -> &mut Vec<SessionSnapshot> {
        &mut self.snapshots
    }

    fn get_compiler_option(&self) -> &CompilerOption {
        &self.compiler_option
    }
}

enum ExpandState {
    Init,
    TryReadMacroName,
    ReadMacroArgs,
}
