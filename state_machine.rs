StateMachine {
    generic_param: Poly { name: ShortString(b"unwrap:3"), kind: Param(0) },
    branches: {
        Data { constructor: { file: File(155), start: 9, end: 19 }, arity: 1 }: Leaves([
            Range({ file: File(155), start: 85, end: 91 }),
        ]),
        Data { constructor: { file: File("@std/option.sdg"), start: 9, end: 15 }, arity: 1 }: Leaves([
            Range({ file: File("@std/option.sdg"), start: 77, end: 83 }),
        ]),
        Data { constructor: { file: File(240), start: 9, end: 18 }, arity: 1 }: Leaves([
            Range({ file: File(240), start: 83, end: 89 }),
        ]),
        Data { constructor: { file: File(187), start: 9, end: 17 }, arity: 1 }: Leaves([
            Range({ file: File(187), start: 81, end: 87 }),
        ]),
        Var: Leaves([
            Range({ file: File(155), start: 85, end: 91 }),
            Range({ file: File(187), start: 81, end: 87 }),
            Range({ file: File(240), start: 83, end: 89 }),
            Range({ file: File("@std/option.sdg"), start: 77, end: 83 }),
        ]),
    },
    default: Leaves([]),
}