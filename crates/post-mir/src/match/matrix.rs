use super::{LiteralType, PatternField, Range};
use crate::Session;
use sodigy_mir::Type;
use sodigy_number::InternedNumber;
use sodigy_span::Span;

/// matrix for `Int`
/// ```ignore
/// Matrix {
///     rows: [MatrixRow { field: [constructor], constructor: Range { Int, -inf..inf } }],
/// }
/// ```
///
/// matrix for `Number`
/// ```ignore
/// Matrix {
///     // We don't care about its denom and numer!
///     rows: [MatrixRow { field: [constructor], constructor: Range { Number, -inf..inf } }],
/// }
/// ```
///
/// matrix for `(Foo, Foo, Int)`, where `struct Foo = { f1: Bool, f2: Int }`
/// ```ignore
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
/// ```ignore
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
            if *constructor_def_span == session.get_lang_item_span("type.Int") ||
               *constructor_def_span == session.get_lang_item_span("type.Char") ||
               *constructor_def_span == session.get_lang_item_span("type.Byte") ||
               *constructor_def_span == session.get_lang_item_span("type.Number") {
                let r#type = if *constructor_def_span == session.get_lang_item_span("type.Int") {
                    LiteralType::Int
                } else if *constructor_def_span == session.get_lang_item_span("type.Char") {
                    LiteralType::Char
                } else if *constructor_def_span == session.get_lang_item_span("type.Byte") {
                    LiteralType::Byte
                } else {
                    LiteralType::Number
                };

                // Invalid ranges (256.. for bytes and 0xd800..0xe000 | 0x110000.. for chars) will be
                // filtered out by `filter_out_invalid_ranges` in `build_tree`.
                vec![MatrixRow {
                    field: vec![PatternField::Constructor],
                    constructor: MatrixConstructor::Range(Range {
                        r#type,
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
                        constructor: MatrixConstructor::DefSpan(constructor_def_span.clone()),
                    },
                    MatrixRow {
                        field: vec![PatternField::ListLength],
                        constructor: MatrixConstructor::Range(Range {
                            r#type: LiteralType::Scalar,
                            lhs: Some(InternedNumber::from_u32(0, true)),
                            lhs_inclusive: true,
                            rhs: None,
                            rhs_inclusive: false,
                        }),
                    },
                    MatrixRow {
                        field: vec![PatternField::ListElements],
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

/// Let's say the patterns in the arms are `[1, 2]`, `[1, 2, 3]`, `[1, 3, 5]`, `[2, 4, 6]` and `_`.
/// First, the decision tree will split the patterns by the lengths of the lists. So `[1, 2]` and
/// `[1, 2, 3]` cannot be in the same sub-matrix. We'll look at the sub-matrix with `[1, 2, 3]`,
/// `[1, 3, 5]`, `[2, 4, 6]` and `_`.
///
/// It's very simple. We can treat the lists like tuples. We do have to worry about rest patterns,
/// and that's why the indexes are signed integers.
pub fn get_list_sub_matrix(
    r#type: &Type,  // of an element, not the list
    field_prefix: &[PatternField],

    // We already know the length of the lists in this submatrix, so
    // we only have to check a few indexes. Read the comment above.
    indexes: &[i32],
    session: &Session,
) -> Vec<MatrixRow> {
    let mut result = vec![];

    for index in indexes.iter() {
        let mut arg_matrix = get_matrix(r#type, session);

        for row in arg_matrix.iter_mut() {
            row.field = vec![
                field_prefix.to_vec(),
                vec![PatternField::ListIndex(*index as i64)],
                row.field.clone(),
            ].concat();
        }

        result.extend(arg_matrix);
    }

    result
}
