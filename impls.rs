PolySolver {
    impls: {
        Range({ file: File("@std/option.sdg"), start: 77, end: 83 }): {
            Poly { name: ShortString(b"unwrap:3"), kind: Return }: GenericParam { def_span: Range({ file: File("@std/option.sdg"), start: 84, end: 85 }), span: None },
            Poly { name: ShortString(b"unwrap:3"), kind: Param(0) }: Data {
                constructor_def_span: { file: File("@std/option.sdg"), start: 9, end: 15 },
                constructor_span: Range({ file: File("@std/option.sdg"), start: 62, end: 68 }),
                args: Some([
                    GenericParam { def_span: Range({ file: File("@std/option.sdg"), start: 84, end: 85 }), span: None },
                ]),
                group_span: Some(Range({ file: File("@std/option.sdg"), start: 68, end: 71 })),
            },
        },
        Range({ file: File(187), start: 81, end: 87 }): {  // MyOption
            Poly { name: ShortString(b"unwrap:3"), kind: Param(0) }: Data {
                constructor_def_span: { file: File(187), start: 9, end: 17 },
                constructor_span: Range({ file: File(187), start: 64, end: 72 }),
                args: Some([
                    GenericParam { def_span: Range({ file: File(187), start: 88, end: 89 }), span: None },
                ]),
                group_span: Some(Range({ file: File(187), start: 72, end: 75 })),
            },
            Poly { name: ShortString(b"unwrap:3"), kind: Return }: GenericParam { def_span: Range({ file: File(187), start: 88, end: 89 }), span: None },
        },
        Range({ file: File(240), start: 83, end: 89 }): {  // OurOption
            Poly { name: ShortString(b"unwrap:3"), kind: Param(0) }: Data {
                constructor_def_span: { file: File(240), start: 9, end: 18 },
                constructor_span: Range({ file: File(240), start: 65, end: 74 }),
                args: Some([GenericParam { def_span: Range({ file: File(240), start: 90, end: 91 }), span: None }]),
                group_span: Some(Range({ file: File(240), start: 74, end: 77 })),
            },
            Poly { name: ShortString(b"unwrap:3"), kind: Return }: GenericParam { def_span: Range({ file: File(240), start: 90, end: 91 }), span: None },
        },
        Range({ file: File(155), start: 85, end: 91 }): {  // YourOption
            Poly { name: ShortString(b"unwrap:3"), kind: Param(0) }: Data {
                constructor_def_span: { file: File(155), start: 9, end: 19 },
                constructor_span: Range({ file: File(155), start: 66, end: 76 }),
                args: Some([GenericParam { def_span: Range({ file: File(155), start: 92, end: 93 }), span: None }]),
                group_span: Some(Range({ file: File(155), start: 76, end: 79 })),
            },
            Poly { name: ShortString(b"unwrap:3"), kind: Return }: GenericParam { def_span: Range({ file: File(155), start: 92, end: 93 }), span: None },
        },
    },
    state_machine: None,
}