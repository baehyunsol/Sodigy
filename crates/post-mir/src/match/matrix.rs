use super::{LiteralType, PatternField, Range};
use crate::Session;
use sodigy_mir::Type;
use sodigy_number::InternedNumber;
use sodigy_span::Span;

/// matrix for `Int`
/// ```
/// Matrix {
///     rows: [MatrixRow { field: [constructor], constructor: Range { Int, -inf..inf } }],
/// }
/// ```
///
/// matrix for `Number`
/// ```
/// Matrix {
///     // We don't care about its denom and numer!
///     rows: [MatrixRow { field: [constructor], constructor: Range { Number, -inf..inf } }],
/// }
/// ```
///
/// matrix for `(Foo, Foo, Int)`, where `struct Foo = { f1: Bool, f2: Int }`
/// ```
/// Matrix {
///     rows: [
///         MatrixRow { field: [constructor], constructor: Tuple(3) },
///         MatrixRow { field: [index(0), constructor], constructor: DefSpan(Foo) },
///         MatrixRow { field: [index(0), name(f1), constructor], constructor: Or(DefSpan(True), DefSpan(False)) },
///
///         // this is empty, but we'll optimize that later
///         MatrixRow { field: [index(0), name(f1), payload], constructor: EnumPayload(Bool) },
///
///         MatrixRow { field: [index(0), name(f2), constructor], constructor: Range { Int, -inf..inf } },
///         MatrixRow { field: [index(1), constructor], constructor: DefSpan(Foo) },
///         MatrixRow { field: [index(1), name(f1), constructor], constructor: Or(DefSpan(True), DefSpan(False)) },
///
///         // this is empty, but we'll optimize that later
///         MatrixRow { field: [index(1), name(f1), payload], constructor: EnumPayload(Bool) },
///
///         MatrixRow { field: [index(1), name(f2), constructor], constructor: Range { Int, -inf..inf } },
///         MatrixRow { field: [index(2), constructor], constructor: Range { Int, -inf..inf } },
///     ],
/// }
/// ```
///
/// matrix for `(Int, Int, Option<Int>)`
/// ```
/// Matrix {
///     rows: [
///         MatrixRow { field: [constructor], constructor: Tuple(3) },
///         MatrixRow { field: [index(0), constructor], constructor: Range { Int, -inf..inf } },
///         MatrixRow { field: [index(1), constructor], constructor: Range { Int, -inf..inf } },
///         MatrixRow { field: [index(2), constructor], constructor: DefSpan(Option) },
///         MatrixRow { field: [index(2), variant], constructor: Or(DefSpan(Some), DefSpan(None)) },
///         MatrixRow { field: [index(2), payload], constructor: EnumPayload(Option) },
///     ],
/// }
/// ```
#[derive(Clone, Debug)]
pub struct MatrixRow {
    pub field: Vec<PatternField>,
    pub constructor: MatrixConstructor,
}

#[derive(Clone, Debug)]
pub enum MatrixConstructor {
    Tuple(usize),
    DefSpan(Span),
    Range(Range),
    ListSubMatrix(Type),
}

pub fn get_matrix(
    r#type: &Type,
    session: &Session,
) -> Vec<MatrixRow> {
    match r#type {
        Type::Data { constructor_def_span, args, .. } => {
            // TODO: It's toooo inefficient to call `get_lang_item_span(...)` everytime.
            if *constructor_def_span == session.get_lang_item_span("type.Int") {
                vec![MatrixRow {
                    field: vec![PatternField::Constructor],
                    constructor: MatrixConstructor::Range(Range {
                        r#type: LiteralType::Int,
                        lhs: None,
                        lhs_inclusive: false,
                        rhs: None,
                        rhs_inclusive: false,
                    }),
                }]
            }

            else if *constructor_def_span == session.get_lang_item_span("type.Tuple") {
                let args = args.as_ref().unwrap();
                let mut result = vec![MatrixRow {
                    field: vec![PatternField::Constructor],
                    constructor: MatrixConstructor::Tuple(args.len()),
                }];

                for (i, arg) in args.iter().enumerate() {
                    let mut arg_matrix = get_matrix(arg, session);

                    for row in arg_matrix.iter_mut() {
                        row.field.insert(0, PatternField::Index(i as i64));
                    }

                    result.extend(arg_matrix);
                }

                result
            }

            else if *constructor_def_span == session.get_lang_item_span("type.List") {
                vec![
                    MatrixRow {
                        field: vec![PatternField::Constructor],
                        constructor: MatrixConstructor::DefSpan(*constructor_def_span),
                    },
                    MatrixRow {
                        field: vec![PatternField::ListLength],
                        constructor: MatrixConstructor::Range(Range {
                            r#type: LiteralType::Int,
                            lhs: Some(InternedNumber::from_u32(0, true)),
                            lhs_inclusive: true,
                            rhs: None,
                            rhs_inclusive: false,
                        }),
                    },
                    MatrixRow {
                        field: vec![PatternField::ListElements],

                        // what here?
                        constructor: MatrixConstructor::ListSubMatrix(args.as_ref().unwrap()[0].clone()),
                    },
                ]
            }

            else {
                todo!()
            }
        },
        Type::Never(_) => todo!(),
        Type::Func { .. } => todo!(),
        Type::GenericParam { .. } |
        Type::Var { .. } |
        Type::GenericArg { .. } |
        Type::Blocked { .. } => panic!("Internal Compiler Error: Type-infer is complete, but I found a type variable!"),
    }
}
