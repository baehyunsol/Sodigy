use crate::{Delim, ParseError, Punct, TokenTree, TokenTreeKind};
use crate::warn::ParseWarning;
use sodigy_err::{ErrorContext, ExpectedToken, SodigyError};
use sodigy_intern::{InternedNumeric, InternedString, InternSession};
use sodigy_lex::LexSession;
use sodigy_number::SodigyNumber;
use sodigy_span::SpanRange;

mod endec;

pub struct ParseSession {
    pub tokens: Vec<TokenTree>,
    pub errors: Vec<ParseError>,
    pub warnings: Vec<ParseWarning>,
    pub interner: InternSession,
    pub has_unexpanded_macros: bool,
}

impl ParseSession {
    pub fn from_lex_session(s: &LexSession) -> Self {
        ParseSession {
            tokens: vec![],
            errors: vec![],
            warnings: vec![],
            interner: s.get_interner().clone(),
            has_unexpanded_macros: false,
        }
    }

    pub fn push_token(&mut self, token: TokenTree) {
        self.tokens.push(token);
    }

    pub fn push_error(&mut self, error: ParseError) {
        self.errors.push(error);
    }

    pub fn push_warning(&mut self, warning: ParseWarning) {
        self.warnings.push(warning);
    }

    pub fn intern_string(&mut self, string: Vec<u8>) -> InternedString {
        self.interner.intern_string(string)
    }

    pub fn intern_numeric(&mut self, numeric: SodigyNumber) -> InternedNumeric {
        self.interner.intern_numeric(numeric)
    }

    pub fn get_tokens(&self) -> &Vec<TokenTree> {
        &self.tokens
    }

    pub fn flush_tokens(&mut self) {
        self.tokens.clear();
    }

    /// EXPENSIVE
    pub fn dump_tokens(&self) -> String {
        self.tokens.iter().map(|t| format!("{t}")).collect::<Vec<String>>().join(" ")
    }

    pub fn get_errors(&self) -> &Vec<ParseError> {
        &self.errors
    }

    pub fn get_warnings(&self) -> &Vec<ParseWarning> {
        &self.warnings
    }

    // TODO: no more `err_if_has_err`
    pub fn err_if_has_err(&self) -> Result<(), ()> {
        if self.errors.is_empty() {
            Ok(())
        }

        else {
            Err(())
        }
    }

    // if it sees `@`, it's not sure whether that's a macro or not
    // if it sees `@[`, that must be a macro!
    pub fn expand_macros(&mut self) -> Result<(), ()> {
        let mut new_tokens = Vec::with_capacity(self.tokens.len());
        let mut errors = vec![];
        let mut curr_state = ExpandState::Init;

        let mut curr_macro_span = SpanRange::dummy(16);
        let mut curr_macro_name = vec![];
        let mut curr_macro_args;

        // TODO: it has too many `clone`s
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
                        curr_macro_name = tokens.clone();
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
                        curr_macro_span = curr_macro_span.merge(curr_span);

                        if let Some(tokens) = self.try_expand_macro(
                            &curr_macro_name,
                            &curr_macro_args,
                        ) {
                            for token in tokens.into_iter() {
                                new_tokens.push(token);
                            }
                        }

                        else {
                            new_tokens.push(TokenTree {
                                kind: TokenTreeKind::Macro {
                                    name: curr_macro_name.clone(),
                                    args: curr_macro_args.clone(),
                                },
                                span: curr_macro_span,
                            });
                            errors.push(
                                ParseError::todo(
                                    "macro",
                                    curr_macro_span,
                                ),
                            );
                            self.has_unexpanded_macros = true;
                        }

                        curr_macro_name.clear();
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
                            ).set_err_context(
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
        self.err_if_has_err()?;

        Ok(())
    }

    pub fn try_expand_macro(
        &self,
        name: &[TokenTree],
        args: &[TokenTree],
    ) -> Option<Vec<TokenTree>> {
        // TODO
        None
    }
}

enum ExpandState {
    Init,
    TryReadMacroName,
    ReadMacroArgs,
}
