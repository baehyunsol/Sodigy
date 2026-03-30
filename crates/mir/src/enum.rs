use crate::{Session, StructField, Type};
use sodigy_error::EnumFieldKind;
use sodigy_hir::{self as hir, FuncPurity, Generic};
use sodigy_span::Span;
use sodigy_string::InternedString;

// `session.types` already has all the necessary information, so this
// struct only has names, which are required if you want to dump mir.
#[derive(Clone, Debug)]
pub struct Enum {
    pub name: InternedString,
    pub name_span: Span,
    pub generics: Vec<Generic>,
    pub variants: Vec<EnumVariant>,
}

#[derive(Clone, Debug)]
pub struct EnumVariant {
    pub name: InternedString,
    pub name_span: Span,
    pub fields: EnumVariantFields,
}

#[derive(Clone, Debug)]
pub enum EnumVariantFields {
    None,
    Tuple(usize),  // number of elements
    Struct(Vec<StructField>),
}

impl Enum {
    pub fn from_hir(hir_enum: &hir::Enum, session: &mut Session) -> Result<Enum, ()> {
        let mut variants = Vec::with_capacity(hir_enum.variants.len());
        let mut has_error = false;
        let enum_type = Type::Data {
            constructor_def_span: hir_enum.name_span.clone(),
            constructor_span: Span::None,
            args: if hir_enum.generics.is_empty() {
                None
            } else {
                Some(hir_enum.generics.iter().map(
                    |generic| Type::GenericParam {
                        def_span: generic.name_span.clone(),
                        span: Span::None,
                    }
                ).collect())
            },
            group_span: if hir_enum.generics.is_empty() { None } else { Some(Span::None) },
        };

        for hir_variant in hir_enum.variants.iter() {
            let fields = match &hir_variant.fields {
                hir::EnumVariantFields::None => {
                    // Type of `Option.None` is `Option<T>`
                    session.types.insert(
                        hir_variant.name_span.clone(),
                        enum_type.clone(),
                    );
                    EnumVariantFields::None
                },
                hir::EnumVariantFields::Tuple(hir_types) => {
                    let mut param_types = Vec::with_capacity(hir_types.len());

                    for hir_type in hir_types.iter() {
                        match Type::from_hir(hir_type, session) {
                            Ok(r#type) => {
                                param_types.push(r#type);
                            },
                            Err(()) => {
                                has_error = true;
                            },
                        }
                    }

                    // Type of `Option.Some` is `Fn(T) -> Option<T>`
                    session.types.insert(
                        hir_variant.name_span.clone(),
                        Type::Func {
                            fn_span: Span::None,
                            group_span: Span::None,
                            params: param_types,
                            r#return: Box::new(enum_type.clone()),
                            purity: FuncPurity::Pure,
                        },
                    );
                    EnumVariantFields::Tuple(hir_types.len())
                },
                hir::EnumVariantFields::Struct(hir_fields) => {
                    let mut fields = Vec::with_capacity(hir_fields.len());
                    let mut param_types = Vec::with_capacity(hir_fields.len());

                    for hir_field in hir_fields.iter() {
                        match hir_field.type_annot.as_ref().map(|hir_type| Type::from_hir(hir_type, session)) {
                            Some(Ok(r#type)) => {
                                param_types.push(r#type);
                            },
                            Some(Err(())) => {
                                has_error = true;
                            },
                            None => {
                                param_types.push(Type::Var {
                                    def_span: hir_field.name_span.clone(),
                                    is_return: false,
                                });
                            },
                        }

                        fields.push(StructField {
                            name: hir_field.name,
                            name_span: hir_field.name_span.clone(),
                            default_value: hir_field.default_value.clone(),
                        });
                    }

                    // `enum MaybePerson = { None, Person { name: String, age: Int } }`
                    // Type of `MaybePerson.Person` is `Fn(String, Int) -> MaybePerson`
                    session.types.insert(
                        hir_variant.name_span.clone(),
                        Type::Func {
                            fn_span: Span::None,
                            group_span: Span::None,
                            params: param_types,
                            r#return: Box::new(enum_type.clone()),
                            purity: FuncPurity::Pure,
                        },
                    );
                    EnumVariantFields::Struct(fields)
                },
            };

            variants.push(EnumVariant {
                name: hir_variant.name,
                name_span: hir_variant.name_span.clone(),
                fields,
            });
        }

        Ok(Enum {
            name: hir_enum.name,
            name_span: hir_enum.name_span.clone(),
            generics: hir_enum.generics.clone(),
            variants,
        })
    }
}

impl From<&EnumVariantFields> for EnumFieldKind {
    fn from(f: &EnumVariantFields) -> EnumFieldKind {
        match f {
            EnumVariantFields::None => EnumFieldKind::None,
            EnumVariantFields::Tuple(_) => EnumFieldKind::Tuple,
            EnumVariantFields::Struct(_) => EnumFieldKind::Struct,
        }
    }
}
