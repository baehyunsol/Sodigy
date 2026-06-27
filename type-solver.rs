let expr = Field {
    lhs: Call {
        func: Dynamic(Field {
            lhs: Call {
                func: Static {
                    def_span: Range({ file: File(95), start: 242, end: 250 }),
                    span: Range({ file: File(95), start: 828, end: 836 }),
                },
                args: [
                    Constant(Number { n: SmallInt { n: 10, is_integer: true }, span: Range({ file: File(95), start: 837, end: 839 }) }),
                ],
                arg_group_span: Range({ file: File(95), start: 836, end: 840 }),
                types: None,
                given_keyword_args: [],
            },
            fields: [
                Name {
                    name: ShortString(b"unwrap"),
                    name_span: Range({ file: File(95), start: 841, end: 847 }),
                    dot_span: Range({ file: File(95), start: 840, end: 841 }),
                    is_from_alias: false,
                },
            ],
            dotfish: [None, None],
        }),
        args: [],
        arg_group_span: Range({ file: File(95), start: 847, end: 849 }),
        types: None, given_keyword_args: [],
    },
    fields: [
        Name {
            name: ShortString(b"unwrap"),
            name_span: Range({ file: File(95), start: 850, end: 856 }),
            dot_span: Range({ file: File(95), start: 849, end: 850 }),
            is_from_alias: false,
        },
    ],
    dotfish: [None, None],
};

let lhs_type = GenericArg { call: Range({ file: File(95), start: 841, end: 847 }), generic: Poly { name: ShortString(b"unwrap:3"), kind: Return } };
let field_type = Blocked { origin: Range({ file: File(95), start: 841, end: 847 }) };
