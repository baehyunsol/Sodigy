use crate::stages::IrStage;
use crate::flag::{Flag, FLAGS};
use sodigy_span::SpanRange;

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub value: TokenValue,
    pub span: SpanRange,
}

#[derive(Clone, Copy, Debug)]
pub enum TokenKind {
    Flag,
    Path,
    Stage,
    Bool,
    None,
}

impl TokenKind {
    pub fn all_possible_values(&self) -> Vec<String> {
        match self {
            TokenKind::Flag => {
                let mut result = vec![];

                for flag in FLAGS.iter() {
                    if let Some(s) = flag.short() {
                        result.push(String::from_utf8(s.to_vec()).unwrap());
                    }

                    result.push(String::from_utf8(flag.long().to_vec()).unwrap());
                }

                result
            },
            TokenKind::Stage => vec![
                "tokens".to_string(),
                "hir".to_string(),
            ],
            TokenKind::Bool => vec![
                "true".to_string(),
                "false".to_string(),
            ],
            TokenKind::Path => vec!["<PATH>".to_string()],
            TokenKind::None => vec![],
        }
    }
}

#[derive(Debug)]
pub enum TokenValue {
    Flag(Flag),
    Path(String),
    Stage(IrStage),
    Bool(bool),
    None,
}

impl TokenValue {
    pub fn try_parse(kind: &TokenKind, buf: &str) -> Option<Self> {
        match kind {
            TokenKind::Path => Some(TokenValue::Path(buf.to_string())),
            TokenKind::Stage => match buf {
                "tokens" => Some(TokenValue::Stage(IrStage::Tokens)),
                "hir" => Some(TokenValue::Stage(IrStage::HighIr)),
                _ => None,
            },
            TokenKind::Bool => match buf {
                "true" => Some(TokenValue::Bool(true)),
                "false" => Some(TokenValue::Bool(false)),
                _ => None,
            },
            TokenKind::None => Some(TokenValue::None),
            _ => None,
        }
    }

    pub fn unwrap_path(&self) -> String {
        match self {
            TokenValue::Path(p) => p.to_string(),
            _ => panic!(),
        }
    }

    pub fn unwrap_flag(&self) -> Flag {
        match self {
            TokenValue::Flag(f) => *f,
            _ => panic!(),
        }
    }

    pub fn unwrap_stage(&self) -> IrStage {
        match self {
            TokenValue::Stage(s) => *s,
            _ => panic!(),
        }
    }

    pub fn unwrap_bool(&self) -> bool {
        match self {
            TokenValue::Bool(b) => *b,
            _ => panic!(),
        }
    }
}
