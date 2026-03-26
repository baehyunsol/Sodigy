use crate::{
    Attribute,
    Expr,
    Field,
    Match,
    MatchArm,
    ParsePatternContext,
    Path,
    Tokens,
    Type,
};
use sodigy_error::{Error, ErrorKind};
use sodigy_span::{RenderableSpan, Span, SpanDeriveKind};
use sodigy_string::{InternedString, intern_string};
use sodigy_token::{Keyword, Punct, Token, TokenKind};
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct Let {
    pub keyword_span: Span,
    pub name: InternedString,
    pub name_span: Span,
    pub type_annot: Option<Type>,
    pub value: Expr,
    pub attribute: Attribute,

    // Hir will lower a pipeline to a block.
    pub from_pipeline: bool,
}

impl<'t, 's> Tokens<'t, 's> {
    pub fn parse_let(&mut self) -> Result<Vec<Let>, Vec<Error>> {
        let prev_cursor = self.cursor;

        match self.parse_let_simple() {
            Ok(r#let) => Ok(vec![r#let]),
            Err(_) => {
                self.cursor = prev_cursor;
                self.parse_let_multiple()
            },
        }
    }

    // Most `let` statements are in this form, so let's do some optimization.
    fn parse_let_simple(&mut self) -> Result<Let, Vec<Error>> {
        let keyword_span = self.match_and_pop(TokenKind::Keyword(Keyword::Let))?.span.clone();
        let (name, name_span) = self.pop_name_and_span(true /* allow_wildcard */)?;

        let type_annot = match self.peek() {
            Some(Token { kind: TokenKind::Punct(Punct::Colon), ..}) => {
                self.cursor += 1;
                Some(self.parse_type()?)
            },
            _ => None,
        };
        self.match_and_pop(TokenKind::Punct(Punct::Assign))?;
        let value = self.parse_expr(true)?;
        self.match_and_pop(TokenKind::Punct(Punct::Semicolon))?;

        Ok(Let {
            keyword_span,
            name,
            name_span,
            type_annot,
            value,
            attribute: Attribute::new(),
            from_pipeline: false,
        })
    }

    // It destructures a pattern into multiple `let` statements.
    //
    // `let Ok(($x, $y)) | Error(($x, _, $y)) = foo();`
    // ->
    // `let tmp = match foo() { Ok(($x, $y)) | Error(($x, _, $y)) => (x, y) };`
    // `let x = tmp._0;`
    // `let y = tmp._1;`
    //
    // `let ($x, _) = foo();`
    // ->
    // `let x = match foo() { ($x, _) => x };`
    //
    // It doesn't do any kind of checks. Hir/Mir will do the checks.
    fn parse_let_multiple(&mut self) -> Result<Vec<Let>, Vec<Error>> {
        let keyword_span = self.match_and_pop(TokenKind::Keyword(Keyword::Let))?.span.clone();
        let pattern = self.parse_pattern(ParsePatternContext::Let)?;

        let type_annot = match self.peek() {
            Some(Token { kind: TokenKind::Punct(Punct::Colon), ..}) => {
                self.cursor += 1;
                Some(self.parse_type()?)
            },
            _ => None,
        };

        self.match_and_pop(TokenKind::Punct(Punct::Assign))?;
        let value = self.parse_expr(true)?;
        self.match_and_pop(TokenKind::Punct(Punct::Semicolon))?;

        let names = pattern.bound_names();

        match names.len() {
            0 => Err(vec![Error {
                kind: ErrorKind::PatternDestructureWithoutNameBindings,
                spans: vec![
                    RenderableSpan {
                        span: keyword_span,
                        auxiliary: false,
                        note: None,
                    },
                    RenderableSpan {
                        span: pattern.error_span_wide(),
                        auxiliary: true,
                        note: Some(String::from("There're no name bindings here.")),
                    },
                ],
                note: None,
            }]),
            1 => Ok(vec![Let {
                keyword_span: keyword_span.clone(),
                name: names[0].0,
                name_span: names[0].1.clone(),
                type_annot,
                value: Expr::Match(Box::new(Match {
                    keyword_span: keyword_span.derive(SpanDeriveKind::LetPattern(0)),
                    scrutinee: Box::new(value),
                    arms: vec![MatchArm {
                        pattern,
                        guard: None,
                        value: Expr::Path(Path {
                            id: names[0].0,
                            id_span: keyword_span.derive(SpanDeriveKind::LetPattern(1)),
                            fields: vec![],
                            dotfish: vec![None],
                        }),
                    }],
                    group_span: keyword_span.derive(SpanDeriveKind::LetPattern(2)),
                    lowered_from_let: true,
                })),
                attribute: Attribute::new(),
                from_pipeline: false,
            }]),
            2.. => {
                // It has to meet some conditions.
                // 1. The name has to be unique.
                //    - It uses `keyword_span.hash()` so that tmp_names are different.
                //    - It uses `@` character so that this name cannot appear in user code.
                // 2. The name has to be shorter than 16 bytes for efficient `intern_string`.
                let tmp_name = format!("@{:012x}", keyword_span.hash() & 0xffff_ffff_ffff);
                let tmp_name = intern_string(tmp_name.as_bytes(), &self.intermediate_dir).unwrap();

                let mut lets = vec![
                    Let {
                        keyword_span: keyword_span.clone(),
                        name: tmp_name,
                        name_span: keyword_span.derive(SpanDeriveKind::LetPattern(0)),
                        type_annot,
                        value: Expr::Match(Box::new(Match {
                            keyword_span: keyword_span.derive(SpanDeriveKind::LetPattern(1)),
                            scrutinee: Box::new(value),
                            arms: vec![MatchArm {
                                pattern,
                                guard: None,
                                value: Expr::Tuple {
                                    elements: names.iter().map(
                                        |(name, name_span)| Expr::Path(Path {
                                            id: *name,
                                            id_span: keyword_span.derive(SpanDeriveKind::LetPattern(2)),
                                            fields: vec![],
                                            dotfish: vec![None],
                                        })
                                    ).collect(),
                                    group_span: keyword_span.derive(SpanDeriveKind::LetPattern(3)),
                                },
                            }],
                            group_span: keyword_span.derive(SpanDeriveKind::LetPattern(4)),
                            lowered_from_let: true,
                        })),
                        attribute: Attribute::new(),
                        from_pipeline: false,
                    },
                ];
                let mut span_derive_index = 5;
                let mut collision_checker = HashSet::new();

                for (i, (name, name_span)) in names.into_iter().enumerate() {
                    // If there are redundant name bindings in a pattern, it only pushes once.
                    // Otherwise, the user will see the same error message twice: one by
                    // `check/pattern.rs` and another by `check.block.rs`.
                    if collision_checker.contains(&name) {
                        continue;
                    }

                    collision_checker.insert(name);
                    lets.push(Let {
                        keyword_span: keyword_span.derive(SpanDeriveKind::LetPattern(span_derive_index)),
                        name,
                        name_span,
                        type_annot: None,
                        value: Expr::Path(Path {
                            id: tmp_name,
                            id_span: keyword_span.derive(SpanDeriveKind::LetPattern(span_derive_index + 1)),
                            fields: vec![Field::Name {
                                name: intern_string(
                                    format!("_{i}").as_bytes(),
                                    &self.intermediate_dir,
                                ).unwrap(),
                                name_span: keyword_span.derive(SpanDeriveKind::LetPattern(span_derive_index + 2)),
                                dot_span: keyword_span.derive(SpanDeriveKind::LetPattern(span_derive_index + 3)),
                                is_from_alias: false,
                            }],
                            dotfish: vec![None, None],
                        }),
                        attribute: Attribute::new(),
                        from_pipeline: false,
                    });
                    span_derive_index += 4;
                }

                Ok(lets)
            },
        }
    }
}
