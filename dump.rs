// name_span: Range { file: Std(12), start: 159, end: 168 }
 fn to_string<T,>(v: T,) -> List<Char> = '\0';
// name_span: Range { file: Std(11), start: 159, end: 169 }
#[built_in]
impure fn random_int() -> Int;
// name_span: Range { file: Std(7), start: 205, end: 214 }
#[built_in]
 fn init_list<T,>() -> List<T>;
// name_span: Range { file: Std(7), start: 500, end: 510 }
#[built_in]
 fn index_list<T,>(ls: List<T>,i: Int,) -> T;
// name_span: Range { file: Std(6), start: 221, end: 225 }
#[built_in]
impure fn exit() -> !;
// name_span: Range { file: Std(6), start: 405, end: 410 }
#[built_in]
 fn panic() -> !;
// name_span: Range { file: Std(6), start: 586, end: 591 }
#[built_in]
impure fn print(v: List<Char>,) -> Int;
// name_span: Range { file: Std(6), start: 780, end: 786 }
#[built_in]
impure fn eprint(v: List<Char>,) -> Int;
// name_span: Range { file: Std(5), start: 246, end: 253 }
#[built_in]
 fn neg_int(a: Int,) -> Int;
// name_span: Range { file: Std(5), start: 339, end: 346 }
#[built_in]
 fn add_int(a: Int,b: Int,) -> Int;
// name_span: Range { file: Std(5), start: 440, end: 447 }
#[built_in]
 fn sub_int(a: Int,b: Int,) -> Int;
// name_span: Range { file: Std(5), start: 541, end: 548 }
#[built_in]
 fn mul_int(a: Int,b: Int,) -> Int;
// name_span: Range { file: Std(5), start: 740, end: 747 }
#[built_in]
 fn div_int(a: Int,b: Int,) -> Int;
// name_span: Range { file: Std(5), start: 841, end: 848 }
#[built_in]
 fn rem_int(a: Int,b: Int,) -> Int;
// name_span: Range { file: Std(5), start: 940, end: 946 }
#[built_in]
 fn lt_int(a: Int,b: Int,) -> Bool;
// name_span: Range { file: Std(5), start: 1039, end: 1045 }
#[built_in]
 fn eq_int(a: Int,b: Int,) -> Bool;
// name_span: Range { file: Std(5), start: 1138, end: 1144 }
#[built_in]
 fn gt_int(a: Int,b: Int,) -> Bool;
// name_span: Range { file: Std(5), start: 1194, end: 1209 }
 fn div_int_wrapper(a: Int,b: Int,) -> Int = {
    if eq_int(
        b,
        0,
        
    ) {
        panic()
    } else {
        div_int(
            a,
            b,
            
        )
    }
};
// name_span: Range { file: Std(9), start: 96, end: 99 }
 fn neg<T,>(x: T,) -> T = '\0';
// name_span: Range { file: Std(9), start: 142, end: 145 }
 fn not(x: Bool,) -> Bool = if x {
    False
} else {
    True
};
// name_span: Range { file: Std(9), start: 322, end: 325 }
 fn add<T,U,V,>(a: T,b: U,) -> V = '\0';
// name_span: Range { file: Std(9), start: 470, end: 473 }
 fn sub<T,U,V,>(a: T,b: U,) -> V = '\0';
// name_span: Range { file: Std(9), start: 618, end: 621 }
 fn mul<T,U,V,>(a: T,b: U,) -> V = '\0';
// name_span: Range { file: Std(9), start: 766, end: 769 }
 fn div<T,U,V,>(a: T,b: U,) -> V = '\0';
// name_span: Range { file: Std(9), start: 914, end: 917 }
 fn rem<T,U,V,>(a: T,b: U,) -> V = '\0';
// name_span: Range { file: Std(9), start: 1070, end: 1075 }
 fn index<T,U,V,>(ls: T,i: U,) -> V = '\0';
// name_span: Range { file: Std(9), start: 1179, end: 1181 }
 fn lt<T,>(lhs: T,rhs: T,) -> Bool = '\0';
// name_span: Range { file: Std(9), start: 1285, end: 1287 }
 fn eq<T,>(lhs: T,rhs: T,) -> Bool = '\0';
// name_span: Range { file: Std(9), start: 1391, end: 1393 }
 fn gt<T,>(lhs: T,rhs: T,) -> Bool = '\0';
// name_span: Range { file: Std(9), start: 1570, end: 1573 }
 fn leq<T,>(lhs: T,rhs: T,) -> Bool = not(gt(
    lhs,
    rhs,
    
));
// name_span: Range { file: Std(9), start: 1767, end: 1770 }
 fn neq<T,>(lhs: T,rhs: T,) -> Bool = not(eq(
    lhs,
    rhs,
    
));
// name_span: Range { file: Std(9), start: 1963, end: 1966 }
 fn geq<T,>(lhs: T,rhs: T,) -> Bool = not(lt(
    lhs,
    rhs,
    
));
// name_span: Range { file: Std(9), start: 2140, end: 2146 }
 fn concat<T,U,V,>(lhs: T,rhs: U,) -> V = '\0';

let session = {
    lets: [], funcs: [
        Func {
            is_pure: true, impure_keyword_span: None, keyword_span: Range {
                file: Std(12), start: 156, end: 158
            }, name: ShortString(b"to_string"), name_span: Range {
                file: Std(12), start: 159, end: 168
            }, generics: [
                Generic {
                    name: ShortString(b"T"), name_span: Range {
                        file: Std(12), start: 169, end: 170
                    }
                }
            ], params: [
                FuncParam {
                    name: ShortString(b"v"), name_span: Range {
                        file: Std(12), start: 172, end: 173
                    }, type_annot: None, default_value: None
                }
            ], type_annot_span: Some(
                Range {
                    file: Std(12), start: 181, end: 187
                }
            ), value: Char {
                ch: 0, span: None
            }, built_in: false, origin: TopLevel
        }, Func {
            is_pure: false, impure_keyword_span: Some(
                Range {
                    file: Std(11), start: 149, end: 155
                }
            ), keyword_span: Range {
                file: Std(11), start: 156, end: 158
            }, name: ShortString(b"random_int"), name_span: Range {
                file: Std(11), start: 159, end: 169
            }, generics: [], params: [], type_annot_span: Some(
                Range {
                    file: Std(11), start: 175, end: 178
                }
            ), value: Char {
                ch: 0, span: None
            }, built_in: true, origin: TopLevel
        }, Func {
            is_pure: true, impure_keyword_span: None, keyword_span: Range {
                file: Std(7), start: 202, end: 204
            }, name: ShortString(b"init_list"), name_span: Range {
                file: Std(7), start: 205, end: 214
            }, generics: [
                Generic {
                    name: ShortString(b"T"), name_span: Range {
                        file: Std(7), start: 215, end: 216
                    }
                }
            ], params: [], type_annot_span: Some(
                Range {
                    file: Std(7), start: 236, end: 239
                }
            ), value: Char {
                ch: 0, span: None
            }, built_in: true, origin: TopLevel
        }, Func {
            is_pure: true, impure_keyword_span: None, keyword_span: Range {
                file: Std(7), start: 497, end: 499
            }, name: ShortString(b"index_list"), name_span: Range {
                file: Std(7), start: 500, end: 510
            }, generics: [
                Generic {
                    name: ShortString(b"T"), name_span: Range {
                        file: Std(7), start: 511, end: 512
                    }
                }
            ], params: [
                FuncParam {
                    name: ShortString(b"ls"), name_span: Range {
                        file: Std(7), start: 514, end: 516
                    }, type_annot: None, default_value: None
                }, FuncParam {
                    name: ShortString(b"i"), name_span: Range {
                        file: Std(7), start: 523, end: 524
                    }, type_annot: None, default_value: None
                }
            ], type_annot_span: Some(
                Range {
                    file: Std(7), start: 534, end: 535
                }
            ), value: Char {
                ch: 0, span: None
            }, built_in: true, origin: TopLevel
        }, Func {
            is_pure: false, impure_keyword_span: Some(
                Range {
                    file: Std(6), start: 211, end: 217
                }
            ), keyword_span: Range {
                file: Std(6), start: 218, end: 220
            }, name: ShortString(b"exit"), name_span: Range {
                file: Std(6), start: 221, end: 225
            }, generics: [], params: [], type_annot_span: Some(
                Range {
                    file: Std(6), start: 231, end: 232
                }
            ), value: Char {
                ch: 0, span: None
            }, built_in: true, origin: TopLevel
        }, Func {
            is_pure: true, impure_keyword_span: None, keyword_span: Range {
                file: Std(6), start: 402, end: 404
            }, name: ShortString(b"panic"), name_span: Range {
                file: Std(6), start: 405, end: 410
            }, generics: [], params: [], type_annot_span: Some(
                Range {
                    file: Std(6), start: 416, end: 417
                }
            ), value: Char {
                ch: 0, span: None
            }, built_in: true, origin: TopLevel
        }, Func {
            is_pure: false, impure_keyword_span: Some(
                Range {
                    file: Std(6), start: 576, end: 582
                }
            ), keyword_span: Range {
                file: Std(6), start: 583, end: 585
            }, name: ShortString(b"print"), name_span: Range {
                file: Std(6), start: 586, end: 591
            }, generics: [], params: [
                FuncParam {
                    name: ShortString(b"v"), name_span: Range {
                        file: Std(6), start: 592, end: 593
                    }, type_annot: None, default_value: None
                }
            ], type_annot_span: Some(
                Range {
                    file: Std(6), start: 606, end: 609
                }
            ), value: Char {
                ch: 0, span: None
            }, built_in: true, origin: TopLevel
        }, Func {
            is_pure: false, impure_keyword_span: Some(
                Range {
                    file: Std(6), start: 770, end: 776
                }
            ), keyword_span: Range {
                file: Std(6), start: 777, end: 779
            }, name: ShortString(b"eprint"), name_span: Range {
                file: Std(6), start: 780, end: 786
            }, generics: [], params: [
                FuncParam {
                    name: ShortString(b"v"), name_span: Range {
                        file: Std(6), start: 787, end: 788
                    }, type_annot: None, default_value: None
                }
            ], type_annot_span: Some(
                Range {
                    file: Std(6), start: 801, end: 804
                }
            ), value: Char {
                ch: 0, span: None
            }, built_in: true, origin: TopLevel
        }, Func {
            is_pure: true, impure_keyword_span: None, keyword_span: Range {
                file: Std(5), start: 243, end: 245
            }, name: ShortString(b"neg_int"), name_span: Range {
                file: Std(5), start: 246, end: 253
            }, generics: [], params: [
                FuncParam {
                    name: ShortString(b"a"), name_span: Range {
                        file: Std(5), start: 254, end: 255
                    }, type_annot: None, default_value: None
                }
            ], type_annot_span: Some(
                Range {
                    file: Std(5), start: 265, end: 268
                }
            ), value: Char {
                ch: 0, span: None
            }, built_in: true, origin: TopLevel
        }, Func {
            is_pure: true, impure_keyword_span: None, keyword_span: Range {
                file: Std(5), start: 336, end: 338
            }, name: ShortString(b"add_int"), name_span: Range {
                file: Std(5), start: 339, end: 346
            }, generics: [], params: [
                FuncParam {
                    name: ShortString(b"a"), name_span: Range {
                        file: Std(5), start: 347, end: 348
                    }, type_annot: None, default_value: None
                }, FuncParam {
                    name: ShortString(b"b"), name_span: Range {
                        file: Std(5), start: 355, end: 356
                    }, type_annot: None, default_value: None
                }
            ], type_annot_span: Some(
                Range {
                    file: Std(5), start: 366, end: 369
                }
            ), value: Char {
                ch: 0, span: None
            }, built_in: true, origin: TopLevel
        }, Func {
            is_pure: true, impure_keyword_span: None, keyword_span: Range {
                file: Std(5), start: 437, end: 439
            }, name: ShortString(b"sub_int"), name_span: Range {
                file: Std(5), start: 440, end: 447
            }, generics: [], params: [
                FuncParam {
                    name: ShortString(b"a"), name_span: Range {
                        file: Std(5), start: 448, end: 449
                    }, type_annot: None, default_value: None
                }, FuncParam {
                    name: ShortString(b"b"), name_span: Range {
                        file: Std(5), start: 456, end: 457
                    }, type_annot: None, default_value: None
                }
            ], type_annot_span: Some(
                Range {
                    file: Std(5), start: 467, end: 470
                }
            ), value: Char {
                ch: 0, span: None
            }, built_in: true, origin: TopLevel
        }, Func {
            is_pure: true, impure_keyword_span: None, keyword_span: Range {
                file: Std(5), start: 538, end: 540
            }, name: ShortString(b"mul_int"), name_span: Range {
                file: Std(5), start: 541, end: 548
            }, generics: [], params: [
                FuncParam {
                    name: ShortString(b"a"), name_span: Range {
                        file: Std(5), start: 549, end: 550
                    }, type_annot: None, default_value: None
                }, FuncParam {
                    name: ShortString(b"b"), name_span: Range {
                        file: Std(5), start: 557, end: 558
                    }, type_annot: None, default_value: None
                }
            ], type_annot_span: Some(
                Range {
                    file: Std(5), start: 568, end: 571
                }
            ), value: Char {
                ch: 0, span: None
            }, built_in: true, origin: TopLevel
        }, Func {
            is_pure: true, impure_keyword_span: None, keyword_span: Range {
                file: Std(5), start: 737, end: 739
            }, name: ShortString(b"div_int"), name_span: Range {
                file: Std(5), start: 740, end: 747
            }, generics: [], params: [
                FuncParam {
                    name: ShortString(b"a"), name_span: Range {
                        file: Std(5), start: 748, end: 749
                    }, type_annot: None, default_value: None
                }, FuncParam {
                    name: ShortString(b"b"), name_span: Range {
                        file: Std(5), start: 756, end: 757
                    }, type_annot: None, default_value: None
                }
            ], type_annot_span: Some(
                Range {
                    file: Std(5), start: 767, end: 770
                }
            ), value: Char {
                ch: 0, span: None
            }, built_in: true, origin: TopLevel
        }, Func {
            is_pure: true, impure_keyword_span: None, keyword_span: Range {
                file: Std(5), start: 838, end: 840
            }, name: ShortString(b"rem_int"), name_span: Range {
                file: Std(5), start: 841, end: 848
            }, generics: [], params: [
                FuncParam {
                    name: ShortString(b"a"), name_span: Range {
                        file: Std(5), start: 849, end: 850
                    }, type_annot: None, default_value: None
                }, FuncParam {
                    name: ShortString(b"b"), name_span: Range {
                        file: Std(5), start: 857, end: 858
                    }, type_annot: None, default_value: None
                }
            ], type_annot_span: Some(
                Range {
                    file: Std(5), start: 868, end: 871
                }
            ), value: Char {
                ch: 0, span: None
            }, built_in: true, origin: TopLevel
        }, Func {
            is_pure: true, impure_keyword_span: None, keyword_span: Range {
                file: Std(5), start: 937, end: 939
            }, name: ShortString(b"lt_int"), name_span: Range {
                file: Std(5), start: 940, end: 946
            }, generics: [], params: [
                FuncParam {
                    name: ShortString(b"a"), name_span: Range {
                        file: Std(5), start: 947, end: 948
                    }, type_annot: None, default_value: None
                }, FuncParam {
                    name: ShortString(b"b"), name_span: Range {
                        file: Std(5), start: 955, end: 956
                    }, type_annot: None, default_value: None
                }
            ], type_annot_span: Some(
                Range {
                    file: Std(5), start: 966, end: 970
                }
            ), value: Char {
                ch: 0, span: None
            }, built_in: true, origin: TopLevel
        }, Func {
            is_pure: true, impure_keyword_span: None, keyword_span: Range {
                file: Std(5), start: 1036, end: 1038
            }, name: ShortString(b"eq_int"), name_span: Range {
                file: Std(5), start: 1039, end: 1045
            }, generics: [], params: [
                FuncParam {
                    name: ShortString(b"a"), name_span: Range {
                        file: Std(5), start: 1046, end: 1047
                    }, type_annot: None, default_value: None
                }, FuncParam {
                    name: ShortString(b"b"), name_span: Range {
                        file: Std(5), start: 1054, end: 1055
                    }, type_annot: None, default_value: None
                }
            ], type_annot_span: Some(
                Range {
                    file: Std(5), start: 1065, end: 1069
                }
            ), value: Char {
                ch: 0, span: None
            }, built_in: true, origin: TopLevel
        }, Func {
            is_pure: true, impure_keyword_span: None, keyword_span: Range {
                file: Std(5), start: 1135, end: 1137
            }, name: ShortString(b"gt_int"), name_span: Range {
                file: Std(5), start: 1138, end: 1144
            }, generics: [], params: [
                FuncParam {
                    name: ShortString(b"a"), name_span: Range {
                        file: Std(5), start: 1145, end: 1146
                    }, type_annot: None, default_value: None
                }, FuncParam {
                    name: ShortString(b"b"), name_span: Range {
                        file: Std(5), start: 1153, end: 1154
                    }, type_annot: None, default_value: None
                }
            ], type_annot_span: Some(
                Range {
                    file: Std(5), start: 1164, end: 1168
                }
            ), value: Char {
                ch: 0, span: None
            }, built_in: true, origin: TopLevel
        }, Func {
            is_pure: true, impure_keyword_span: None, keyword_span: Range {
                file: Std(5), start: 1191, end: 1193
            }, name: ShortString(b"div_int_wrapper"), name_span: Range {
                file: Std(5), start: 1194, end: 1209
            }, generics: [], params: [
                FuncParam {
                    name: ShortString(b"a"), name_span: Range {
                        file: Std(5), start: 1210, end: 1211
                    }, type_annot: None, default_value: None
                }, FuncParam {
                    name: ShortString(b"b"), name_span: Range {
                        file: Std(5), start: 1218, end: 1219
                    }, type_annot: None, default_value: None
                }
            ], type_annot_span: Some(
                Range {
                    file: Std(5), start: 1229, end: 1232
                }
            ), value: Block(
                Block {
                    group_span: Range {
                        file: Std(5), start: 1235, end: 1351
                    }, lets: [], asserts: [], value: If(
                        If {
                            if_span: Range {
                                file: Std(5), start: 1241, end: 1243
                            }, cond: Call {
                                func: Static {
                                    def_span: Range {
                                        file: Std(5), start: 1039, end: 1045
                                    }, span: Range {
                                        file: Std(5), start: 1246, end: 1248
                                    }
                                }, args: [
                                    Ident(
                                        IdentWithOrigin {
                                            id: ShortString(b"b"), span: Range {
                                                file: Std(5), start: 1244, end: 1245
                                            }, origin: FuncParam { index: 1 }, def_span: Range {
                                                file: Std(5), start: 1218, end: 1219
                                            }
                                        }
                                    ), Number {
                                        n: InternedNumber {
                                            value: SmallInt(0), is_integer: true
                                        }, span: Range {
                                            file: Std(5), start: 1249, end: 1250
                                        }
                                    }
                                ], arg_group_span: Derived {
                                    kind: Trivial, file: Std(5), start: 1244, end: 1250
                                }, generic_defs: [], given_keyword_arguments: []
                            }, else_span: Range {
                                file: Std(5), start: 1315, end: 1319
                            }, true_value: Call {
                                func: Static {
                                    def_span: Range {
                                        file: Std(6), start: 405, end: 410
                                    }, span: Range {
                                        file: Std(5), start: 1296, end: 1301
                                    }
                                }, args: [], arg_group_span: Range {
                                    file: Std(5), start: 1301, end: 1303
                                }, generic_defs: [], given_keyword_arguments: []
                            }, true_group_span: Range {
                                file: Std(5), start: 1251, end: 1309
                            }, false_value: Call {
                                func: Static {
                                    def_span: Range {
                                        file: Std(5), start: 740, end: 747
                                    }, span: Range {
                                        file: Std(5), start: 1330, end: 1337
                                    }
                                }, args: [
                                    Ident(
                                        IdentWithOrigin {
                                            id: ShortString(b"a"), span: Range {
                                                file: Std(5), start: 1338, end: 1339
                                            }, origin: FuncParam { index: 0 }, def_span: Range {
                                                file: Std(5), start: 1210, end: 1211
                                            }
                                        }
                                    ), Ident(
                                        IdentWithOrigin {
                                            id: ShortString(b"b"), span: Range {
                                                file: Std(5), start: 1341, end: 1342
                                            }, origin: FuncParam { index: 1 }, def_span: Range {
                                                file: Std(5), start: 1218, end: 1219
                                            }
                                        }
                                    )
                                ], arg_group_span: Range {
                                    file: Std(5), start: 1337, end: 1343
                                }, generic_defs: [], given_keyword_arguments: []
                            }, false_group_span: Range {
                                file: Std(5), start: 1320, end: 1349
                            }, from_short_circuit: None
                        }
                    )
                }
            ), built_in: false, origin: TopLevel
        }, Func {
            is_pure: true, impure_keyword_span: None, keyword_span: Range {
                file: Std(9), start: 93, end: 95
            }, name: ShortString(b"neg"), name_span: Range {
                file: Std(9), start: 96, end: 99
            }, generics: [
                Generic {
                    name: ShortString(b"T"), name_span: Range {
                        file: Std(9), start: 100, end: 101
                    }
                }
            ], params: [
                FuncParam {
                    name: ShortString(b"x"), name_span: Range {
                        file: Std(9), start: 103, end: 104
                    }, type_annot: None, default_value: None
                }
            ], type_annot_span: Some(
                Range {
                    file: Std(9), start: 112, end: 113
                }
            ), value: Char {
                ch: 0, span: None
            }, built_in: false, origin: TopLevel
        }, Func {
            is_pure: true, impure_keyword_span: None, keyword_span: Range {
                file: Std(9), start: 139, end: 141
            }, name: ShortString(b"not"), name_span: Range {
                file: Std(9), start: 142, end: 145
            }, generics: [], params: [
                FuncParam {
                    name: ShortString(b"x"), name_span: Range {
                        file: Std(9), start: 146, end: 147
                    }, type_annot: None, default_value: None
                }
            ], type_annot_span: Some(
                Range {
                    file: Std(9), start: 158, end: 162
                }
            ), value: If(
                If {
                    if_span: Range {
                        file: Std(9), start: 165, end: 167
                    }, cond: Ident(
                        IdentWithOrigin {
                            id: ShortString(b"x"), span: Range {
                                file: Std(9), start: 168, end: 169
                            }, origin: FuncParam { index: 0 }, def_span: Range {
                                file: Std(9), start: 146, end: 147
                            }
                        }
                    ), else_span: Range {
                        file: Std(9), start: 185, end: 189
                    }, true_value: Ident(
                        IdentWithOrigin {
                            id: ShortString(b"False"), span: Range {
                                file: Std(9), start: 177, end: 182
                            }, origin: Foreign {
                                kind: EnumVariant {
                                    parent: Range {
                                        file: Std(0), start: 35, end: 39
                                    }
                                }
                            }, def_span: Range {
                                file: Std(0), start: 136, end: 141
                            }
                        }
                    ), true_group_span: Range {
                        file: Std(9), start: 170, end: 184
                    }, false_value: Ident(
                        IdentWithOrigin {
                            id: ShortString(b"True"), span: Range {
                                file: Std(9), start: 197, end: 201
                            }, origin: Foreign {
                                kind: EnumVariant {
                                    parent: Range {
                                        file: Std(0), start: 35, end: 39
                                    }
                                }
                            }, def_span: Range {
                                file: Std(0), start: 86, end: 90
                            }
                        }
                    ), false_group_span: Range {
                        file: Std(9), start: 190, end: 203
                    }, from_short_circuit: None
                }
            ), built_in: false, origin: TopLevel
        }, Func {
            is_pure: true, impure_keyword_span: None, keyword_span: Range {
                file: Std(9), start: 319, end: 321
            }, name: ShortString(b"add"), name_span: Range {
                file: Std(9), start: 322, end: 325
            }, generics: [
                Generic {
                    name: ShortString(b"T"), name_span: Range {
                        file: Std(9), start: 326, end: 327
                    }
                }, Generic {
                    name: ShortString(b"U"), name_span: Range {
                        file: Std(9), start: 329, end: 330
                    }
                }, Generic {
                    name: ShortString(b"V"), name_span: Range {
                        file: Std(9), start: 332, end: 333
                    }
                }
            ], params: [
                FuncParam {
                    name: ShortString(b"a"), name_span: Range {
                        file: Std(9), start: 335, end: 336
                    }, type_annot: None, default_value: None
                }, FuncParam {
                    name: ShortString(b"b"), name_span: Range {
                        file: Std(9), start: 341, end: 342
                    }, type_annot: None, default_value: None
                }
            ], type_annot_span: Some(
                Range {
                    file: Std(9), start: 350, end: 351
                }
            ), value: Char {
                ch: 0, span: None
            }, built_in: false, origin: TopLevel
        }, Func {
            is_pure: true, impure_keyword_span: None, keyword_span: Range {
                file: Std(9), start: 467, end: 469
            }, name: ShortString(b"sub"), name_span: Range {
                file: Std(9), start: 470, end: 473
            }, generics: [
                Generic {
                    name: ShortString(b"T"), name_span: Range {
                        file: Std(9), start: 474, end: 475
                    }
                }, Generic {
                    name: ShortString(b"U"), name_span: Range {
                        file: Std(9), start: 477, end: 478
                    }
                }, Generic {
                    name: ShortString(b"V"), name_span: Range {
                        file: Std(9), start: 480, end: 481
                    }
                }
            ], params: [
                FuncParam {
                    name: ShortString(b"a"), name_span: Range {
                        file: Std(9), start: 483, end: 484
                    }, type_annot: None, default_value: None
                }, FuncParam {
                    name: ShortString(b"b"), name_span: Range {
                        file: Std(9), start: 489, end: 490
                    }, type_annot: None, default_value: None
                }
            ], type_annot_span: Some(
                Range {
                    file: Std(9), start: 498, end: 499
                }
            ), value: Char {
                ch: 0, span: None
            }, built_in: false, origin: TopLevel
        }, Func {
            is_pure: true, impure_keyword_span: None, keyword_span: Range {
                file: Std(9), start: 615, end: 617
            }, name: ShortString(b"mul"), name_span: Range {
                file: Std(9), start: 618, end: 621
            }, generics: [
                Generic {
                    name: ShortString(b"T"), name_span: Range {
                        file: Std(9), start: 622, end: 623
                    }
                }, Generic {
                    name: ShortString(b"U"), name_span: Range {
                        file: Std(9), start: 625, end: 626
                    }
                }, Generic {
                    name: ShortString(b"V"), name_span: Range {
                        file: Std(9), start: 628, end: 629
                    }
                }
            ], params: [
                FuncParam {
                    name: ShortString(b"a"), name_span: Range {
                        file: Std(9), start: 631, end: 632
                    }, type_annot: None, default_value: None
                }, FuncParam {
                    name: ShortString(b"b"), name_span: Range {
                        file: Std(9), start: 637, end: 638
                    }, type_annot: None, default_value: None
                }
            ], type_annot_span: Some(
                Range {
                    file: Std(9), start: 646, end: 647
                }
            ), value: Char {
                ch: 0, span: None
            }, built_in: false, origin: TopLevel
        }, Func {
            is_pure: true, impure_keyword_span: None, keyword_span: Range {
                file: Std(9), start: 763, end: 765
            }, name: ShortString(b"div"), name_span: Range {
                file: Std(9), start: 766, end: 769
            }, generics: [
                Generic {
                    name: ShortString(b"T"), name_span: Range {
                        file: Std(9), start: 770, end: 771
                    }
                }, Generic {
                    name: ShortString(b"U"), name_span: Range {
                        file: Std(9), start: 773, end: 774
                    }
                }, Generic {
                    name: ShortString(b"V"), name_span: Range {
                        file: Std(9), start: 776, end: 777
                    }
                }
            ], params: [
                FuncParam {
                    name: ShortString(b"a"), name_span: Range {
                        file: Std(9), start: 779, end: 780
                    }, type_annot: None, default_value: None
                }, FuncParam {
                    name: ShortString(b"b"), name_span: Range {
                        file: Std(9), start: 785, end: 786
                    }, type_annot: None, default_value: None
                }
            ], type_annot_span: Some(
                Range {
                    file: Std(9), start: 794, end: 795
                }
            ), value: Char {
                ch: 0, span: None
            }, built_in: false, origin: TopLevel
        }, Func {
            is_pure: true, impure_keyword_span: None, keyword_span: Range {
                file: Std(9), start: 911, end: 913
            }, name: ShortString(b"rem"), name_span: Range {
                file: Std(9), start: 914, end: 917
            }, generics: [
                Generic {
                    name: ShortString(b"T"), name_span: Range {
                        file: Std(9), start: 918, end: 919
                    }
                }, Generic {
                    name: ShortString(b"U"), name_span: Range {
                        file: Std(9), start: 921, end: 922
                    }
                }, Generic {
                    name: ShortString(b"V"), name_span: Range {
                        file: Std(9), start: 924, end: 925
                    }
                }
            ], params: [
                FuncParam {
                    name: ShortString(b"a"), name_span: Range {
                        file: Std(9), start: 927, end: 928
                    }, type_annot: None, default_value: None
                }, FuncParam {
                    name: ShortString(b"b"), name_span: Range {
                        file: Std(9), start: 933, end: 934
                    }, type_annot: None, default_value: None
                }
            ], type_annot_span: Some(
                Range {
                    file: Std(9), start: 942, end: 943
                }
            ), value: Char {
                ch: 0, span: None
            }, built_in: false, origin: TopLevel
        }, Func {
            is_pure: true, impure_keyword_span: None, keyword_span: Range {
                file: Std(9), start: 1067, end: 1069
            }, name: ShortString(b"index"), name_span: Range {
                file: Std(9), start: 1070, end: 1075
            }, generics: [
                Generic {
                    name: ShortString(b"T"), name_span: Range {
                        file: Std(9), start: 1076, end: 1077
                    }
                }, Generic {
                    name: ShortString(b"U"), name_span: Range {
                        file: Std(9), start: 1079, end: 1080
                    }
                }, Generic {
                    name: ShortString(b"V"), name_span: Range {
                        file: Std(9), start: 1082, end: 1083
                    }
                }
            ], params: [
                FuncParam {
                    name: ShortString(b"ls"), name_span: Range {
                        file: Std(9), start: 1085, end: 1087
                    }, type_annot: None, default_value: None
                }, FuncParam {
                    name: ShortString(b"i"), name_span: Range {
                        file: Std(9), start: 1092, end: 1093
                    }, type_annot: None, default_value: None
                }
            ], type_annot_span: Some(
                Range {
                    file: Std(9), start: 1101, end: 1102
                }
            ), value: Char {
                ch: 0, span: None
            }, built_in: false, origin: TopLevel
        }, Func {
            is_pure: true, impure_keyword_span: None, keyword_span: Range {
                file: Std(9), start: 1176, end: 1178
            }, name: ShortString(b"lt"), name_span: Range {
                file: Std(9), start: 1179, end: 1181
            }, generics: [
                Generic {
                    name: ShortString(b"T"), name_span: Range {
                        file: Std(9), start: 1182, end: 1183
                    }
                }
            ], params: [
                FuncParam {
                    name: ShortString(b"lhs"), name_span: Range {
                        file: Std(9), start: 1185, end: 1188
                    }, type_annot: None, default_value: None
                }, FuncParam {
                    name: ShortString(b"rhs"), name_span: Range {
                        file: Std(9), start: 1193, end: 1196
                    }, type_annot: None, default_value: None
                }
            ], type_annot_span: Some(
                Range {
                    file: Std(9), start: 1204, end: 1208
                }
            ), value: Char {
                ch: 0, span: None
            }, built_in: false, origin: TopLevel
        }, Func {
            is_pure: true, impure_keyword_span: None, keyword_span: Range {
                file: Std(9), start: 1282, end: 1284
            }, name: ShortString(b"eq"), name_span: Range {
                file: Std(9), start: 1285, end: 1287
            }, generics: [
                Generic {
                    name: ShortString(b"T"), name_span: Range {
                        file: Std(9), start: 1288, end: 1289
                    }
                }
            ], params: [
                FuncParam {
                    name: ShortString(b"lhs"), name_span: Range {
                        file: Std(9), start: 1291, end: 1294
                    }, type_annot: None, default_value: None
                }, FuncParam {
                    name: ShortString(b"rhs"), name_span: Range {
                        file: Std(9), start: 1299, end: 1302
                    }, type_annot: None, default_value: None
                }
            ], type_annot_span: Some(
                Range {
                    file: Std(9), start: 1310, end: 1314
                }
            ), value: Char {
                ch: 0, span: None
            }, built_in: false, origin: TopLevel
        }, Func {
            is_pure: true, impure_keyword_span: None, keyword_span: Range {
                file: Std(9), start: 1388, end: 1390
            }, name: ShortString(b"gt"), name_span: Range {
                file: Std(9), start: 1391, end: 1393
            }, generics: [
                Generic {
                    name: ShortString(b"T"), name_span: Range {
                        file: Std(9), start: 1394, end: 1395
                    }
                }
            ], params: [
                FuncParam {
                    name: ShortString(b"lhs"), name_span: Range {
                        file: Std(9), start: 1397, end: 1400
                    }, type_annot: None, default_value: None
                }, FuncParam {
                    name: ShortString(b"rhs"), name_span: Range {
                        file: Std(9), start: 1405, end: 1408
                    }, type_annot: None, default_value: None
                }
            ], type_annot_span: Some(
                Range {
                    file: Std(9), start: 1416, end: 1420
                }
            ), value: Char {
                ch: 0, span: None
            }, built_in: false, origin: TopLevel
        }, Func {
            is_pure: true, impure_keyword_span: None, keyword_span: Range {
                file: Std(9), start: 1567, end: 1569
            }, name: ShortString(b"leq"), name_span: Range {
                file: Std(9), start: 1570, end: 1573
            }, generics: [
                Generic {
                    name: ShortString(b"T"), name_span: Range {
                        file: Std(9), start: 1574, end: 1575
                    }
                }
            ], params: [
                FuncParam {
                    name: ShortString(b"lhs"), name_span: Range {
                        file: Std(9), start: 1577, end: 1580
                    }, type_annot: None, default_value: None
                }, FuncParam {
                    name: ShortString(b"rhs"), name_span: Range {
                        file: Std(9), start: 1585, end: 1588
                    }, type_annot: None, default_value: None
                }
            ], type_annot_span: Some(
                Range {
                    file: Std(9), start: 1596, end: 1600
                }
            ), value: Call {
                func: Static {
                    def_span: Range {
                        file: Std(9), start: 142, end: 145
                    }, span: Range {
                        file: Std(9), start: 1603, end: 1604
                    }
                }, args: [
                    Call {
                        func: Static {
                            def_span: Range {
                                file: Std(9), start: 1391, end: 1393
                            }, span: Range {
                                file: Std(9), start: 1604, end: 1606
                            }
                        }, args: [
                            Ident(
                                IdentWithOrigin {
                                    id: ShortString(b"lhs"), span: Range {
                                        file: Std(9), start: 1607, end: 1610
                                    }, origin: FuncParam { index: 0 }, def_span: Range {
                                        file: Std(9), start: 1577, end: 1580
                                    }
                                }
                            ), Ident(
                                IdentWithOrigin {
                                    id: ShortString(b"rhs"), span: Range {
                                        file: Std(9), start: 1612, end: 1615
                                    }, origin: FuncParam { index: 1 }, def_span: Range {
                                        file: Std(9), start: 1585, end: 1588
                                    }
                                }
                            )
                        ], arg_group_span: Range {
                            file: Std(9), start: 1606, end: 1616
                        }, generic_defs: [
                            Range {
                                file: Std(9), start: 1394, end: 1395
                            }
                        ], given_keyword_arguments: []
                    }
                ], arg_group_span: Derived {
                    kind: Trivial, file: Std(9), start: 1604, end: 1616
                }, generic_defs: [], given_keyword_arguments: []
            }, built_in: false, origin: TopLevel
        }, Func {
            is_pure: true, impure_keyword_span: None, keyword_span: Range {
                file: Std(9), start: 1764, end: 1766
            }, name: ShortString(b"neq"), name_span: Range {
                file: Std(9), start: 1767, end: 1770
            }, generics: [
                Generic {
                    name: ShortString(b"T"), name_span: Range {
                        file: Std(9), start: 1771, end: 1772
                    }
                }
            ], params: [
                FuncParam {
                    name: ShortString(b"lhs"), name_span: Range {
                        file: Std(9), start: 1774, end: 1777
                    }, type_annot: None, default_value: None
                }, FuncParam {
                    name: ShortString(b"rhs"), name_span: Range {
                        file: Std(9), start: 1782, end: 1785
                    }, type_annot: None, default_value: None
                }
            ], type_annot_span: Some(
                Range {
                    file: Std(9), start: 1793, end: 1797
                }
            ), value: Call {
                func: Static {
                    def_span: Range {
                        file: Std(9), start: 142, end: 145
                    }, span: Range {
                        file: Std(9), start: 1800, end: 1801
                    }
                }, args: [
                    Call {
                        func: Static {
                            def_span: Range {
                                file: Std(9), start: 1285, end: 1287
                            }, span: Range {
                                file: Std(9), start: 1801, end: 1803
                            }
                        }, args: [
                            Ident(
                                IdentWithOrigin {
                                    id: ShortString(b"lhs"), span: Range {
                                        file: Std(9), start: 1804, end: 1807
                                    }, origin: FuncParam { index: 0 }, def_span: Range {
                                        file: Std(9), start: 1774, end: 1777
                                    }
                                }
                            ), Ident(
                                IdentWithOrigin {
                                    id: ShortString(b"rhs"), span: Range {
                                        file: Std(9), start: 1809, end: 1812
                                    }, origin: FuncParam { index: 1 }, def_span: Range {
                                        file: Std(9), start: 1782, end: 1785
                                    }
                                }
                            )
                        ], arg_group_span: Range {
                            file: Std(9), start: 1803, end: 1813
                        }, generic_defs: [
                            Range {
                                file: Std(9), start: 1288, end: 1289
                            }
                        ], given_keyword_arguments: []
                    }
                ], arg_group_span: Derived {
                    kind: Trivial, file: Std(9), start: 1801, end: 1813
                }, generic_defs: [], given_keyword_arguments: []
            }, built_in: false, origin: TopLevel
        }, Func {
            is_pure: true, impure_keyword_span: None, keyword_span: Range {
                file: Std(9), start: 1960, end: 1962
            }, name: ShortString(b"geq"), name_span: Range {
                file: Std(9), start: 1963, end: 1966
            }, generics: [
                Generic {
                    name: ShortString(b"T"), name_span: Range {
                        file: Std(9), start: 1967, end: 1968
                    }
                }
            ], params: [
                FuncParam {
                    name: ShortString(b"lhs"), name_span: Range {
                        file: Std(9), start: 1970, end: 1973
                    }, type_annot: None, default_value: None
                }, FuncParam {
                    name: ShortString(b"rhs"), name_span: Range {
                        file: Std(9), start: 1978, end: 1981
                    }, type_annot: None, default_value: None
                }
            ], type_annot_span: Some(
                Range {
                    file: Std(9), start: 1989, end: 1993
                }
            ), value: Call {
                func: Static {
                    def_span: Range {
                        file: Std(9), start: 142, end: 145
                    }, span: Range {
                        file: Std(9), start: 1996, end: 1997
                    }
                }, args: [
                    Call {
                        func: Static {
                            def_span: Range {
                                file: Std(9), start: 1179, end: 1181
                            }, span: Range {
                                file: Std(9), start: 1997, end: 1999
                            }
                        }, args: [
                            Ident(
                                IdentWithOrigin {
                                    id: ShortString(b"lhs"), span: Range {
                                        file: Std(9), start: 2000, end: 2003
                                    }, origin: FuncParam { index: 0 }, def_span: Range {
                                        file: Std(9), start: 1970, end: 1973
                                    }
                                }
                            ), Ident(
                                IdentWithOrigin {
                                    id: ShortString(b"rhs"), span: Range {
                                        file: Std(9), start: 2005, end: 2008
                                    }, origin: FuncParam { index: 1 }, def_span: Range {
                                        file: Std(9), start: 1978, end: 1981
                                    }
                                }
                            )
                        ], arg_group_span: Range {
                            file: Std(9), start: 1999, end: 2009
                        }, generic_defs: [
                            Range {
                                file: Std(9), start: 1182, end: 1183
                            }
                        ], given_keyword_arguments: []
                    }
                ], arg_group_span: Derived {
                    kind: Trivial, file: Std(9), start: 1997, end: 2009
                }, generic_defs: [], given_keyword_arguments: []
            }, built_in: false, origin: TopLevel
        }, Func {
            is_pure: true, impure_keyword_span: None, keyword_span: Range {
                file: Std(9), start: 2137, end: 2139
            }, name: ShortString(b"concat"), name_span: Range {
                file: Std(9), start: 2140, end: 2146
            }, generics: [
                Generic {
                    name: ShortString(b"T"), name_span: Range {
                        file: Std(9), start: 2147, end: 2148
                    }
                }, Generic {
                    name: ShortString(b"U"), name_span: Range {
                        file: Std(9), start: 2150, end: 2151
                    }
                }, Generic {
                    name: ShortString(b"V"), name_span: Range {
                        file: Std(9), start: 2153, end: 2154
                    }
                }
            ], params: [
                FuncParam {
                    name: ShortString(b"lhs"), name_span: Range {
                        file: Std(9), start: 2156, end: 2159
                    }, type_annot: None, default_value: None
                }, FuncParam {
                    name: ShortString(b"rhs"), name_span: Range {
                        file: Std(9), start: 2164, end: 2167
                    }, type_annot: None, default_value: None
                }
            ], type_annot_span: Some(
                Range {
                    file: Std(9), start: 2175, end: 2176
                }
            ), value: Char {
                ch: 0, span: None
            }, built_in: false, origin: TopLevel
        }
    ], asserts: []
};