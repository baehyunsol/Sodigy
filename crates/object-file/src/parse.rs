use crate::{
    Assert,
    BasicBlock,
    Code,
    CodeKind,
    Entry,
    ObjectFile,
    Terminator,
};
use sodigy_bytecode::{
    Bytecode,
    ExprHash,
    GlobalLabel,
    InternedValue,
    LocalLabel,
    Memory,
    SSA,
    Value,
};
use sodigy_error::{Error, ErrorKind, FuncEffect};
use sodigy_file::File;
use sodigy_number::{BigInt, neg_bi};
use sodigy_span::{RenderableSpan, Span, SpanHash};
use sodigy_string::InternedString;
use sodigy_token::Delim;
use std::collections::hash_map::{Entry as HashMapEntry, HashMap};

mod lex;
mod token;

use lex::lex;
use token::{
    Command,
    Keyword,
    SectionName,
    Token,
    TokenKind,
};

pub fn parse(bytes: &[u8], file: File, intermediate_dir: &str) -> Result<ObjectFile, Error> {
    let tokens = lex(bytes, file, intermediate_dir)?;
    let mut cursor = 0;

    pop_section_name(SectionName::Data, &tokens, &mut cursor)?;
    let data = parse_data_section(&tokens, &mut cursor)?;

    pop_section_name(SectionName::Code, &tokens, &mut cursor)?;
    let code = parse_code_section(&tokens, &mut cursor, intermediate_dir)?;

    let entry = match tokens.get(cursor) {
        Some(Token { kind: TokenKind::Section(sect @ (SectionName::Asserts | SectionName::Main)), .. }) => {
            cursor += 1;
            pop_punct(b':', &tokens, &mut cursor)?;
            parse_entry_section(&tokens, &mut cursor, *sect, intermediate_dir)?
        },
        Some(t) => todo!(),
        None => todo!(),
    };

    Ok(ObjectFile { data, code, entry })
}

fn parse_data_section(tokens: &[Token], cursor: &mut usize) -> Result<HashMap<ExprHash, Value>, Error> {
    let mut result = HashMap::new();

    loop {
        match tokens.get(*cursor) {
            Some(Token { kind: TokenKind::InternedValue(expr_hash), .. }) => {
                *cursor += 1;
                pop_punct(b'=', tokens, cursor)?;
                let value = parse_value(tokens, cursor)?;
                pop_punct(b';', tokens, cursor)?;
                result.insert(ExprHash::from_raw(*expr_hash), value);
            },
            Some(Token { kind: TokenKind::Section(SectionName::Code), .. }) => break,
            Some(b) => {
                return Err(Error {
                    kind: ErrorKind::ObjectFileUnexpectedToken {
                        expected: Some(String::from("%Ixxxx")),
                        got: b.kind.render(),
                    },
                    spans: b.span.simple_error(),
                    note: None,
                });
            },
            None => todo!(),
        }
    }

    Ok(result)
}

fn parse_code_section(tokens: &[Token], cursor: &mut usize, intermediate_dir: &str) -> Result<HashMap<GlobalLabel, Code>, Error> {
    let mut result = HashMap::new();

    loop {
        match tokens.get(*cursor) {
            Some(Token { kind: TokenKind::Section(_), .. }) => break,
            _ => {
                let code = parse_code(tokens, cursor, intermediate_dir)?;
                result.insert(code.label, code);
            },
        }
    }

    Ok(result)
}

fn parse_entry_section(
    tokens: &[Token],
    cursor: &mut usize,
    section: SectionName,
    intermediate_dir: &str,
) -> Result<Entry, Error> {
    match section {
        SectionName::Asserts => {
            let mut asserts = vec![];

            loop {
                let name = match tokens.get(*cursor) {
                    Some(Token { kind: TokenKind::String(name), .. }) => {
                        *cursor += 1;
                        name.unintern_or_default(intermediate_dir)
                    },
                    Some(t) => {
                        return Err(Error {
                            kind: ErrorKind::ObjectFileUnexpectedToken {
                                expected: Some(TokenKind::String(InternedString::dummy()).render()),
                                got: t.kind.render(),
                            },
                            spans: t.span.simple_error(),
                            note: None,
                        });
                    },
                    None => break,
                };
                pop_punct(b':', tokens, cursor)?;
                let label = match tokens.get(*cursor) {
                    Some(Token { kind: TokenKind::GlobalLabel(label), .. }) => {
                        *cursor += 1;
                        GlobalLabel::new(SpanHash(*label))
                    },
                    s => panic!("{s:?}"),
                };

                // TODO: we have to encode the `is_std` field.
                asserts.push(Assert { name, label, is_std: false });
            }

            Ok(Entry::Asserts(asserts))
        },
        SectionName::Main => match tokens.get(*cursor) {
            Some(Token { kind: TokenKind::GlobalLabel(label), .. }) => Ok(Entry::Main(GlobalLabel::new(SpanHash(*label)))),
            s => panic!("{s:?}"),
        },
        _ => unreachable!(),
    }
}

fn parse_code(tokens: &[Token], cursor: &mut usize, intermediate_dir: &str) -> Result<Code, Error> {
    let mut effect: Option<FuncEffect> = None;
    let mut name: Option<String> = None;
    let mut kind: Option<CodeKind> = None;

    loop {
        match tokens.get(*cursor) {
            Some(Token { kind: TokenKind::Group { delim: Delim::Decorator, tokens }, .. }) => match tokens.get(0) {
                Some(Token { kind: TokenKind::Keyword(Keyword::Effect), .. }) => match tokens.get(1) {
                    Some(Token { kind: TokenKind::Group { delim: Delim::Parenthesis, tokens }, .. }) => match tokens.get(0) {
                        Some(Token { kind: TokenKind::Keyword(k @ (Keyword::Fn | Keyword::Proc | Keyword::NdetFn | Keyword::NdetProc)), .. }) => match (k, effect) {
                            (_, Some(_)) => todo!(),  // error: `#[effect(..)]` appears twice
                            (Keyword::Fn, _) => { effect = Some(FuncEffect::Fn); },
                            (Keyword::Proc, _) => { effect = Some(FuncEffect::Proc); },
                            (Keyword::NdetFn, _) => { effect = Some(FuncEffect::NdetFn); },
                            (Keyword::NdetProc, _) => { effect = Some(FuncEffect::NdetProc); },
                            _ => unreachable!(),
                        },
                        s => panic!("{s:?}"),
                    },
                    s => panic!("{s:?}"),
                },
                Some(Token { kind: TokenKind::Keyword(Keyword::Name), .. }) => match tokens.get(1) {
                    Some(Token { kind: TokenKind::Group { delim: Delim::Parenthesis, tokens }, .. }) => match tokens.get(0) {
                        Some(Token { kind: TokenKind::String(s), .. }) => match &name {
                            Some(_) => todo!(),  // error: `#[name(..)]` appears twice
                            None => { name = Some(s.unintern_or_default(intermediate_dir)); },
                        },
                        s => panic!("{s:?}"),
                    },
                    s => panic!("{s:?}"),
                },
                Some(Token { kind: TokenKind::Keyword(Keyword::Kind), .. }) => match tokens.get(1) {
                    Some(Token { kind: TokenKind::Group { delim: Delim::Parenthesis, tokens }, .. }) => match tokens.get(0) {
                        Some(Token { kind: TokenKind::Keyword(k @ (Keyword::Fn | Keyword::Let | Keyword::Assert)), .. }) => match (k, kind) {
                            (_, Some(_)) => todo!(),  // error: `#[kind(..)]` appears twice
                            (Keyword::Fn, _) => { kind = Some(CodeKind::Fn); },
                            (Keyword::Let, _) => { kind = Some(CodeKind::Let); },
                            (Keyword::Assert, _) => { kind = Some(CodeKind::Assert); },
                            _ => unreachable!(),
                        },
                        s => panic!("{s:?}"),
                    },
                    s => panic!("{s:?}"),
                },
                s => panic!("{s:?}"),
            },
            Some(Token { kind: TokenKind::Keyword(Keyword::Code), .. }) => {
                *cursor += 1;
                break;
            },
            s => panic!("{s:?}"),
        }

        *cursor += 1;
    }

    let global_label = match tokens.get(*cursor) {
        Some(Token { kind: TokenKind::GlobalLabel(label), .. }) => {
            *cursor += 1;
            *label
        },
        s => panic!("{s:?}"),
    };

    let params: Option<usize> = match (tokens.get(*cursor), tokens.get(*cursor + 1)) {
        (Some(Token { kind: TokenKind::Punct(b':'), .. }), _) => {
            *cursor += 1;
            None
        },
        (
            Some(Token { kind: TokenKind::Group { delim: Delim::Parenthesis, tokens }, .. }),
            Some(Token { kind: TokenKind::Punct(b':'), .. }),
        ) => {
            *cursor += 2;
            let params = parse_ssa_list(tokens)?;
            Some(params.len())
        },
        s => panic!("{s:?}"),
    };

    let mut basic_blocks: HashMap<LocalLabel, BasicBlock> = HashMap::new();
    let mut ssa_spans: HashMap<SSA, &Span> = HashMap::new();

    loop {
        match tokens.get(*cursor) {
            Some(Token { kind: TokenKind::Group { delim: Delim::Decorator, .. }, .. }) |
            Some(Token { kind: TokenKind::Section(_), .. }) => break,
            _ => {
                let basic_block = parse_basic_block(tokens, cursor, &mut ssa_spans)?;
                basic_blocks.insert(basic_block.label, basic_block);
            },
        }
    }

    Ok(Code {
        label: GlobalLabel::new(SpanHash(global_label)),
        span: None,
        kind: kind.unwrap(),
        name: name.unwrap(),
        params,
        effect: effect.unwrap(),
        basic_blocks,
    })
}

fn parse_value(tokens: &[Token], cursor: &mut usize) -> Result<Value, Error> {
    match tokens.get(*cursor) {
        Some(Token { kind: TokenKind::Punct(b'-'), span: neg_span }) => {
            *cursor += 1;

            match tokens.get(*cursor) {
                Some(Token { kind: TokenKind::Scalar(_) | TokenKind::FuncPointer(_), span: n_span }) => todo!(),
                Some(Token { kind: TokenKind::Int(n), .. }) => {
                    *cursor += 1;
                    let (is_neg, nums) = neg_bi(n.is_neg, &n.nums);
                    Ok(Value::Int(BigInt { is_neg, nums }))
                },
                _ => todo!(),
            }
        },
        Some(Token { kind: TokenKind::Scalar(n), .. }) => {
            *cursor += 1;
            Ok(Value::Scalar(*n))
        },
        Some(Token { kind: TokenKind::Int(n), .. }) => {
            *cursor += 1;
            Ok(Value::Int(n.clone()))
        },
        Some(Token { kind: TokenKind::Group { delim, tokens }, .. }) => {
            let mut values = vec![];
            let mut inner_cursor = 0;

            loop {
                if inner_cursor >= tokens.len() {
                    break;
                }

                values.push(parse_value(tokens, &mut inner_cursor)?);

                if inner_cursor >= tokens.len() {
                    break;
                }

                pop_punct(b',', tokens, &mut inner_cursor)?;
            }

            *cursor += 1;

            match delim {
                Delim::Bracket => Ok(Value::List(values)),
                Delim::Brace => Ok(Value::Compound(values)),
                _ => unreachable!(),  // lexer prevents this
            }
        },
        t => panic!("TODO: {t:?}"),
    }
}

fn parse_basic_block<'t>(
    tokens: &'t [Token],
    cursor: &mut usize,
    ssa_spans: &mut HashMap<SSA, &'t Span>,
) -> Result<BasicBlock, Error> {
    let label = match (tokens.get(*cursor), tokens.get(*cursor + 1), tokens.get(*cursor + 2)) {
        (
            Some(Token { kind: TokenKind::Keyword(Keyword::Label), .. }),
            Some(Token { kind: TokenKind::LocalLabel(label), .. }),
            Some(Token { kind: TokenKind::Punct(b':'), .. }),
        ) => {
            *cursor += 3;
            LocalLabel::new(*label)
        },
        s => panic!("{s:?}"),
    };
    let mut code = vec![];
    let mut terminator = None;

    loop {
        match tokens.get(*cursor) {
            Some(Token { kind: TokenKind::SSA(_) | TokenKind::Punct(b'*'), .. }) => {
                let dst = match (
                    tokens.get(*cursor),
                    tokens.get(*cursor + 1),
                    tokens.get(*cursor + 2),
                ) {
                    (
                        Some(Token { kind: TokenKind::SSA(ssa), span: ssa_span }),
                        Some(Token { kind: TokenKind::Punct(b'='), .. }),
                        _,
                    ) => {
                        let ssa = SSA::from_u32(*ssa);
                        *cursor += 2;

                        match ssa_spans.entry(ssa) {
                            HashMapEntry::Occupied(e) => {
                                return Err(Error {
                                    kind: ErrorKind::RepeatedSSA(ssa.to_u32()),
                                    spans: vec![
                                        RenderableSpan {
                                            span: (*e.get()).clone(),
                                            auxiliary: true,
                                            note: Some(format!("{ssa} is already assigned here.")),
                                        },
                                        RenderableSpan {
                                            span: ssa_span.clone(),
                                            auxiliary: false,
                                            note: Some(format!("You're trying to assign {ssa} again here.")),
                                        },
                                    ],
                                    note: None,
                                });
                            },
                            HashMapEntry::Vacant(e) => {
                                e.insert(ssa_span);
                            },
                        }

                        Memory::SSA(ssa)
                    },
                    (
                        Some(Token { kind: TokenKind::Punct(b'*'), .. }),
                        Some(Token { kind: TokenKind::SSA(ssa), .. }),
                        Some(Token { kind: TokenKind::Punct(b'='), .. }),
                    ) => {
                        *cursor += 3;
                        Memory::Heap { ptr: SSA::from_u32(*ssa), offset: 0 }
                    },
                    (
                        Some(Token { kind: TokenKind::Punct(b'*'), .. }),
                        Some(Token { kind: TokenKind::Group { delim: Delim::Parenthesis, tokens }, span: group_span }),
                        Some(Token { kind: TokenKind::Punct(b'='), .. }),
                    ) => {
                        *cursor += 3;
                        let (ptr, offset) = parse_heap_offset(tokens, group_span)?;
                        Memory::Heap { ptr, offset }
                    },
                    (
                        Some(Token { kind: TokenKind::SSA(ssa), .. }),
                        Some(Token { kind: TokenKind::Group { delim: Delim::Bracket, tokens }, span: group_span }),
                        Some(Token { kind: TokenKind::Punct(b'='), .. }),
                    ) => {
                        *cursor += 3;
                        let offset = parse_single_scalar(tokens, false, group_span)?;
                        Memory::List { ptr: SSA::from_u32(*ssa), offset }
                    },
                    s => panic!("{s:?}"),
                };
                let c = match (tokens.get(*cursor), tokens.get(*cursor + 1)) {
                    (Some(Token { kind: TokenKind::InternedValue(v), .. }), _) => {
                        *cursor += 1;
                        Bytecode::Const {
                            value: InternedValue::Interned(ExprHash::from_raw(*v)),
                            dst,
                            debug_info: None,
                        }
                    },
                    (Some(Token { kind: TokenKind::Scalar(n), .. }), _) => {
                        *cursor += 1;
                        Bytecode::Const {
                            value: InternedValue::Scalar(*n),
                            dst,
                            debug_info: None,
                        }
                    },
                    (Some(Token { kind: TokenKind::FuncPointer(p), .. }), _) => {
                        *cursor += 1;
                        Bytecode::Const {
                            value: InternedValue::FuncPointer(SpanHash(*p)),
                            dst,
                            debug_info: None,
                        }
                    },
                    (
                        Some(Token { kind: TokenKind::SSA(ssa), .. }),
                        Some(Token { kind: TokenKind::Group { delim: Delim::Bracket, tokens }, span: group_span }),
                    ) => {
                        *cursor += 2;
                        let offset = parse_single_scalar(tokens, false, group_span)?;
                        Bytecode::Move {
                            src: Memory::List { ptr: SSA::from_u32(*ssa), offset },
                            dst,
                        }
                    },
                    (
                        Some(Token { kind: TokenKind::SSA(ssa), .. }),
                        Some(Token { kind: TokenKind::Group { delim: Delim::Parenthesis, tokens }, .. }),
                    ) => {
                        *cursor += 2;
                        let args = parse_ssa_list(tokens)?;
                        Bytecode::CallDynamic {
                            func: SSA::from_u32(*ssa),
                            args,
                            dst: Some(dst),
                            debug_info: None,
                            effect: None,
                        }
                    },
                    (Some(Token { kind: TokenKind::SSA(ssa), .. }), _) => {
                        *cursor += 1;
                        Bytecode::Move {
                            src: Memory::SSA(SSA::from_u32(*ssa)),
                            dst,
                        }
                    },
                    (
                        Some(Token { kind: TokenKind::Punct(b'*'), .. }),
                        Some(Token { kind: TokenKind::SSA(ssa), .. }),
                    ) => {
                        *cursor += 2;
                        Bytecode::Move {
                            src: Memory::Heap { ptr: SSA::from_u32(*ssa), offset: 0 },
                            dst,
                        }
                    },
                    (
                        Some(Token { kind: TokenKind::Punct(b'*'), .. }),
                        Some(Token { kind: TokenKind::Group { delim: Delim::Parenthesis, tokens }, span: group_span }),
                    ) => {
                        *cursor += 2;
                        let (ptr, offset) = parse_heap_offset(tokens, group_span)?;
                        Bytecode::Move {
                            src: Memory::Heap { ptr, offset },
                            dst,
                        }
                    },
                    (
                        Some(Token { kind: TokenKind::Command(Command::Phi), span: phi_span }),
                        Some(Token { kind: TokenKind::Group { delim: Delim::Parenthesis, tokens }, .. }),
                    ) => {
                        *cursor += 2;
                        let args = parse_ssa_list(tokens)?;
                        let (x, y) = match args.len() {
                            2 => (args[0], args[1]),
                            _ => {
                                return Err(Error {
                                    kind: ErrorKind::ObjectFileWrongNumberOfArgs {
                                        expected: 2,
                                        got: args.len(),
                                    },
                                    spans: phi_span.simple_error(),
                                    note: None,
                                });
                            },
                        };

                        Bytecode::Phi {
                            pair: (x, y),
                            dst,
                        }
                    },
                    (
                        Some(Token { kind: TokenKind::Command(command @ (Command::InitList | Command::InitTuple)), .. }),
                        Some(Token { kind: TokenKind::Group { delim: Delim::Parenthesis, tokens }, span: group_span }),
                    ) => {
                        *cursor += 2;
                        let elements = parse_single_scalar(tokens, true, group_span)? as usize;

                        if *command == Command::InitList {
                            Bytecode::InitList {
                                elements,
                                dst,
                                debug_info: None,
                            }
                        } else {
                            Bytecode::InitTuple {
                                elements,
                                dst,
                                debug_info: None,
                            }
                        }
                    },
                    (
                        Some(Token { kind: TokenKind::Command(Command::LoadGlobal), .. }),
                        Some(Token { kind: TokenKind::Group { delim: Delim::Parenthesis, tokens }, .. }),
                    ) => {
                        *cursor += 2;
                        let src = match (tokens.get(0), tokens.get(1)) {
                            (Some(Token { kind: TokenKind::GlobalValue(val), .. }), None) => GlobalLabel::new(SpanHash(*val)),
                            s => panic!("{s:?}"),
                        };
                        let dst = match dst {
                            Memory::SSA(ssa) => ssa,

                            // TODO: I have to throw an error with a proper span, but I'm tooooo
                            // lazy to write the code...
                            _ => panic!("{dst:?}"),
                        };

                        Bytecode::LoadGlobal { src, dst }
                    },
                    (
                        Some(Token { kind: TokenKind::Command(command), span: command_span }),
                        Some(Token { kind: TokenKind::Group { delim: Delim::Parenthesis, tokens }, .. }),
                    ) => {
                        *cursor += 2;
                        let intrinsic = match command.try_into_intrinsic() {
                            Some(i) => i,
                            None => {
                                return Err(Error {
                                    kind: ErrorKind::InvalidIntrinsic(format!("{command:?}")),
                                    spans: command_span.simple_error(),
                                    note: None,
                                });
                            },
                        };
                        let args = parse_ssa_list(tokens)?;

                        Bytecode::Intrinsic {
                            intrinsic,
                            args,
                            dst,
                            debug_info: None,
                        }
                    },
                    (Some(Token { kind: TokenKind::Command(command), .. }), _) => {
                        *cursor += 1;
                        todo!()
                    },
                    (
                        Some(Token { kind: TokenKind::GlobalLabel(label), .. }),
                        Some(Token { kind: TokenKind::Group { delim: Delim::Parenthesis, tokens }, .. }),
                    ) => {
                        *cursor += 2;
                        let args = parse_ssa_list(tokens)?;

                        Bytecode::Call {
                            func: GlobalLabel::new(SpanHash(*label)),
                            args,
                            dst: Some(dst),
                            debug_info: None,
                        }
                    },
                    (Some(t @ Token { span, .. }), _) => {
                        return Err(Error {
                            kind: ErrorKind::Todo { id: 306994 },
                            spans: span.simple_error(),
                            note: Some(format!("{t:?}")),
                        });
                    },
                    s => panic!("{s:?}"),
                };
                pop_punct(b';', tokens, cursor)?;

                code.push(c);
            },
            Some(Token { kind: TokenKind::Keyword(Keyword::Return), .. }) => {
                *cursor += 1;

                match tokens.get(*cursor) {
                    Some(Token { kind: TokenKind::SSA(ssa), .. }) => {
                        *cursor += 1;
                        pop_punct(b';', tokens, cursor)?;
                        terminator = Some(Terminator::Return(SSA::from_u32(*ssa)));
                        break;
                    },
                    Some(Token { kind: TokenKind::GlobalLabel(label), .. }) => {
                        *cursor += 1;
                        let args = match tokens.get(*cursor) {
                            Some(Token { kind: TokenKind::Group { delim: Delim::Parenthesis, tokens }, .. }) => parse_ssa_list(tokens)?,
                            Some(t) => {
                                return Err(Error {
                                    kind: ErrorKind::ObjectFileUnexpectedToken {
                                        expected: Some(TokenKind::Group { delim: Delim::Parenthesis, tokens: vec![] }.render()),
                                        got: t.kind.render(),
                                    },
                                    spans: t.span.simple_error(),
                                    note: Some(String::from("You forgot the arguments!")),
                                });
                            },
                            None => {
                                return Err(Error {
                                    kind: ErrorKind::ObjectFileUnexpectedEog {
                                        expected: Some(TokenKind::Group { delim: Delim::Parenthesis, tokens: vec![] }.render()),
                                    },
                                    spans: vec![],
                                    note: None,
                                });
                            },
                        };
                        *cursor += 1;
                        pop_punct(b';', tokens, cursor)?;
                        terminator = Some(Terminator::TailCall {
                            func: GlobalLabel::new(SpanHash(*label)),
                            args,
                        });
                        break;
                    },
                    s => panic!("{s:?}"),
                }
            },
            Some(Token { kind: TokenKind::Keyword(Keyword::If), .. }) => {
                *cursor += 1;

                match tokens.get(*cursor) {
                    Some(Token { kind: TokenKind::SSA(cond), .. }) => {
                        *cursor += 1;
                        let cond = SSA::from_u32(*cond);
                        let t_label = parse_jump_in_brace(tokens, cursor)?;
                        pop_keyword(Keyword::Else, tokens, cursor)?;
                        let f_label = parse_jump_in_brace(tokens, cursor)?;
                        terminator = Some(Terminator::JumpIf {
                            value: cond,
                            t: t_label,
                            f: f_label,
                        });
                        break;
                    },
                    Some(Token { kind: TokenKind::Punct(b'!'), span }) => {
                        *cursor += 1;

                        match (tokens.get(*cursor), tokens.get(*cursor + 1)) {
                            (
                                Some(Token { kind: TokenKind::Command(Command::IsInit), .. }),
                                Some(Token { kind: TokenKind::Group { delim: Delim::Parenthesis, tokens: group_tokens }, .. }),
                            ) => {
                                *cursor += 2;
                                let global = match (group_tokens.get(0), group_tokens.get(1)) {
                                    (Some(Token { kind: TokenKind::GlobalValue(label), .. }), None) => GlobalLabel::new(SpanHash(*label)),
                                    s => panic!("{s:?}"),
                                };

                                // We have to parse the curly brace, but I'm toooo lazy to write the
                                // code... let's just assume that there's no syntax error :)
                                *cursor += 1;

                                pop_keyword(Keyword::Jump, tokens, cursor)?;
                                let local = match tokens.get(*cursor) {
                                    Some(Token { kind: TokenKind::LocalLabel(label), .. }) => LocalLabel::new(*label),
                                    s => panic!("{s:?}"),
                                };
                                *cursor += 1;
                                pop_punct(b';', tokens, cursor)?;
                                terminator = Some(Terminator::TryInitGlobal {
                                    global,
                                    label: local,
                                });
                                break;
                            },
                            (Some(t), _) => {
                                return Err(Error {
                                    kind: ErrorKind::Todo { id: 534465 },
                                    spans: t.span.simple_error(),
                                    note: None,
                                });
                            },
                            (None, _) => {
                                return Err(Error {
                                    kind: ErrorKind::ObjectFileUnexpectedEog {
                                        expected: Some(TokenKind::Command(Command::IsInit).render()),
                                    },
                                    spans: span.simple_error(),
                                    note: None,
                                });
                            },
                        }
                    },
                    s => panic!("{s:?}"),
                }
            },
            Some(Token { kind: TokenKind::Keyword(Keyword::Jump), .. }) => {
                *cursor += 1;

                match tokens.get(*cursor) {
                    Some(Token { kind: TokenKind::LocalLabel(label), .. }) => {
                        *cursor += 1;
                        pop_punct(b';', tokens, cursor)?;
                        terminator = Some(Terminator::Jump(LocalLabel::new(*label)));
                        break;
                    },
                    s => panic!("{s:?}"),
                }
            },
            Some(Token { kind: TokenKind::Command(command), span }) => {
                *cursor += 1;
                let c = match *command {
                    Command::Breakpoint => Bytecode::Breakpoint,
                    Command::StoreGlobal => match tokens.get(*cursor) {
                        Some(Token { kind: TokenKind::Group { delim: Delim::Parenthesis, tokens }, .. }) => match (
                            tokens.get(0),
                            tokens.get(1),
                            tokens.get(2),
                            tokens.get(3),
                        ) {
                            (
                                Some(Token { kind: TokenKind::SSA(ssa), .. }),
                                Some(Token { kind: TokenKind::Punct(b','), .. }),
                                Some(Token { kind: TokenKind::GlobalValue(global), .. }),
                                None,
                            ) => {
                                *cursor += 1;
                                Bytecode::StoreGlobal {
                                    src: SSA::from_u32(*ssa),
                                    dst: GlobalLabel::new(SpanHash(*global)),
                                }
                            },
                            s => panic!("{s:?}"),
                        },
                        Some(t) => {
                            return Err(Error {
                                kind: ErrorKind::ObjectFileUnexpectedToken {
                                    expected: Some(TokenKind::Group { delim: Delim::Parenthesis, tokens: vec![] }.render()),
                                    got: t.kind.render(),
                                },
                                spans: t.span.simple_error(),
                                note: None,
                            });
                        },
                        None => {
                            return Err(Error {
                                kind: ErrorKind::ObjectFileUnexpectedEog {
                                    expected: Some(TokenKind::Group { delim: Delim::Parenthesis, tokens: vec![] }.render()),
                                },
                                spans: span.simple_error(),
                                note: None,
                            });
                        },
                    },
                    s => panic!("{s:?}"),
                };

                pop_punct(b';', tokens, cursor)?;
                code.push(c);
            },
            s => panic!("{s:?}"),
        }
    }

    Ok(BasicBlock {
        label,
        code,
        terminator: terminator.unwrap(),
        terminator_debug_info: None,
    })
}

// It has to consume all the tokens.
fn parse_ssa_list(tokens: &[Token]) -> Result<Vec<SSA>, Error> {
    let mut cursor = 0;
    let mut result = vec![];

    loop {
        match tokens.get(cursor) {
            Some(Token { kind: TokenKind::SSA(ssa), .. }) => {
                cursor += 1;
                result.push(SSA::from_u32(*ssa));
            },
            Some(t) => {
                return Err(Error {
                    kind: ErrorKind::ObjectFileUnexpectedToken {
                        expected: Some(TokenKind::SSA(1).render()),
                        got: t.kind.render(),
                    },
                    spans: t.span.simple_error(),
                    note: None,
                });
            },
            None => break,
        }

        match (tokens.get(cursor), tokens.get(cursor + 1)) {
            (Some(Token { kind: TokenKind::Punct(b','), .. }), None) | (None, _) => break,
            (Some(Token { kind: TokenKind::Punct(b','), .. }), _) => { cursor += 1; },
            (Some(t), _) => {
                return Err(Error {
                    kind: ErrorKind::ObjectFileUnexpectedToken {
                        expected: Some(TokenKind::Punct(b',').render()),
                        got: t.kind.render(),
                    },
                    spans: t.span.simple_error(),
                    note: None,
                });
            },
        }
    }

    Ok(result)
}

// It has to consume all the tokens.
fn parse_single_scalar(
    tokens: &[Token],
    allow_trailing_comma: bool,
    eof_error_span: &Span,
) -> Result<u32, Error> {
    match (tokens.get(0), tokens.get(1)) {
        (
            Some(Token { kind: TokenKind::Scalar(n), .. }),
            trail @ (None | Some(Token { kind: TokenKind::Punct(b','), .. })),
        ) => {
            if let Some(t) = trail && !allow_trailing_comma {
                Err(Error {
                    kind: ErrorKind::ObjectFileUnexpectedToken {
                        expected: None,
                        got: t.kind.render(),
                    },
                    spans: t.span.simple_error(),
                    note: None,
                })
            } else {
                Ok(*n)
            }
        },
        (Some(Token { kind: TokenKind::Scalar(_), .. }), Some(t)) => Err(Error {
            kind: ErrorKind::ObjectFileUnexpectedToken {
                expected: None,
                got: t.kind.render(),
            },
            spans: t.span.simple_error(),
            note: None,
        }),
        (Some(t), _) => Err(Error {
            kind: ErrorKind::ObjectFileUnexpectedToken {
                expected: Some(TokenKind::Scalar(1).render()),
                got: t.kind.render(),
            },
            spans: t.span.simple_error(),
            note: None,
        }),
        (None, _) => Err(Error {
            kind: ErrorKind::ObjectFileUnexpectedEog {
                expected: Some(TokenKind::Scalar(1).render()),
            },
            spans: eof_error_span.simple_error(),
            note: None,
        }),
    }
}

fn parse_jump_in_brace(tokens: &[Token], cursor: &mut usize) -> Result<LocalLabel, Error> {
    match tokens.get(*cursor) {
        Some(Token { kind: TokenKind::Group { delim: Delim::Brace, tokens }, .. }) => {
            *cursor += 1;
            let mut inner_cursor = 0;
            pop_keyword(Keyword::Jump, tokens, &mut inner_cursor)?;

            match tokens.get(inner_cursor) {
                Some(Token { kind: TokenKind::LocalLabel(label), .. }) => {
                    let label = LocalLabel::new(*label);
                    inner_cursor += 1;
                    pop_punct(b';', tokens, &mut inner_cursor)?;
                    return Ok(label);
                },
                s => panic!("{s:?}"),
            }
        },
        Some(t) => {
            return Err(Error {
                kind: ErrorKind::ObjectFileUnexpectedToken {
                    expected: Some(TokenKind::Group { delim: Delim::Brace, tokens: vec![] }.render()),
                    got: t.kind.render(),
                },
                spans: t.span.simple_error(),
                note: None,
            });
        },
        None => {
            return Err(Error {
                kind: ErrorKind::ObjectFileUnexpectedEog {
                    expected: Some(TokenKind::Group { delim: Delim::Brace, tokens: vec![] }.render()),
                },
                spans: vec![],
                note: None,
            });
        },
    }
}

// It has to consume all the tokens.
fn parse_heap_offset(tokens: &[Token], eof_error_span: &Span) -> Result<(SSA, u32), Error> {
    let ssa = match tokens.get(0) {
        Some(Token { kind: TokenKind::SSA(ssa), .. }) => SSA::from_u32(*ssa),
        Some(t) => {
            return Err(Error {
                kind: ErrorKind::ObjectFileUnexpectedToken {
                    expected: Some(TokenKind::SSA(1).render()),
                    got: t.kind.render(),
                },
                spans: t.span.simple_error(),
                note: None,
            });
        },
        None => {
            return Err(Error {
                kind: ErrorKind::ObjectFileUnexpectedEog {
                    expected: Some(TokenKind::SSA(0).render()),
                },
                spans: eof_error_span.simple_error(),
                note: None,
            });
        },
    };
    let offset = match tokens.get(1) {
        None => 0,
        Some(Token { kind: TokenKind::Punct(b'+'), .. }) => match (tokens.get(2), tokens.get(3)) {
            (Some(Token { kind: TokenKind::Scalar(n), .. }), None) => *n,
            (Some(Token { kind: TokenKind::Scalar(_), .. }), Some(t)) => {
                return Err(Error {
                    kind: ErrorKind::ObjectFileUnexpectedToken {
                        expected: None,
                        got: t.kind.render(),
                    },
                    spans: t.span.simple_error(),
                    note: None,
                });
            },
            s => panic!("{s:?}"),
        },
        Some(t) => {
            let is_minus = matches!(&t.kind, TokenKind::Punct(b'-'));
            return Err(Error {
                kind: ErrorKind::ObjectFileUnexpectedToken {
                    expected: Some(TokenKind::Punct(b'+').render()),
                    got: t.kind.render(),
                },
                spans: t.span.simple_error(),
                note: if is_minus {
                    Some(String::from("An offset cannot be minus."))
                } else {
                    None
                },
            });
        },
    };

    Ok((ssa, offset))
}

fn pop_keyword(keyword: Keyword, tokens: &[Token], cursor: &mut usize) -> Result<(), Error> {
    match tokens.get(*cursor) {
        Some(Token { kind: TokenKind::Keyword(k), .. }) if *k == keyword => {
            *cursor += 1;
            Ok(())
        },
        Some(b) => Err(Error {
            kind: ErrorKind::ObjectFileUnexpectedToken {
                expected: Some(TokenKind::Keyword(keyword).render()),
                got: b.kind.render(),
            },
            spans: b.span.simple_error(),
            note: None,
        }),
        None => Err(Error {
            kind: ErrorKind::ObjectFileUnexpectedEog {
                expected: Some(TokenKind::Keyword(keyword).render()),
            },
            spans: vec![],
            note: None,
        }),
    }
}

fn pop_punct(punct: u8, tokens: &[Token], cursor: &mut usize) -> Result<(), Error> {
    match tokens.get(*cursor) {
        Some(Token { kind: TokenKind::Punct(b), .. }) if *b == punct => {
            *cursor += 1;
            Ok(())
        },
        Some(b) => Err(Error {
            kind: ErrorKind::ObjectFileUnexpectedToken {
                expected: Some(TokenKind::Punct(punct).render()),
                got: b.kind.render(),
            },
            spans: b.span.simple_error(),
            note: None,
        }),
        None => Err(Error {
            kind: ErrorKind::ObjectFileUnexpectedEog {
                expected: Some(TokenKind::Punct(punct).render()),
            },
            spans: vec![],
            note: None,
        }),
    }
}

fn pop_section_name(section: SectionName, tokens: &[Token], cursor: &mut usize) -> Result<(), Error> {
    match tokens.get(*cursor) {
        Some(Token { kind: TokenKind::Section(s), .. }) if *s == section => match tokens.get(*cursor + 1) {
            Some(Token { kind: TokenKind::Punct(b':'), .. }) => {
                *cursor += 2;
                Ok(())
            },
            Some(b) => Err(Error {
                kind: ErrorKind::ObjectFileUnexpectedToken {
                    expected: Some(TokenKind::Section(section).render()),
                    got: b.kind.render(),
                },
                spans: b.span.simple_error(),
                note: None,
            }),
            None => Err(Error {
                kind: ErrorKind::ObjectFileUnexpectedEog {
                    expected: Some(TokenKind::Section(section).render()),
                },
                spans: vec![],
                note: None,
            }),
        },
        Some(b) => Err(Error {
            kind: ErrorKind::ObjectFileUnexpectedToken {
                expected: Some(TokenKind::Section(section).render()),
                got: b.kind.render(),
            },
            spans: b.span.simple_error(),
            note: None,
        }),
        None => Err(Error {
            kind: ErrorKind::ObjectFileUnexpectedEog {
                expected: Some(TokenKind::Section(section).render()),
            },
            spans: vec![],
            note: None,
        }),
    }
}

