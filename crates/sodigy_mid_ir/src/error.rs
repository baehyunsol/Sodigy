use crate::func::LocalValue;
use smallvec::{SmallVec, smallvec};
use sodigy_error::{
    ExtraErrorInfo,
    RenderError,
    SodigyError,
    SodigyErrorKind,
    Stage,
    concat_commas,
    substr_edit_distance,
};
use sodigy_high_ir::{self as hir, NameBindingType};
use sodigy_intern::{InternedString, InternSession, unintern_string};
use sodigy_parse::IdentWithSpan;
use sodigy_span::SpanRange;

mod endec;

pub struct MirError {
    kind: MirErrorKind,
    spans: SmallVec<[SpanRange; 1]>,
    extra: ExtraErrorInfo,
}

impl MirError {
    pub fn recursive_local_value(local_value: &LocalValue, expr_span: SpanRange) -> Self {
        MirError {
            kind: MirErrorKind::RecursiveLocalValue { name: local_value.name.id(), name_binding_type: local_value.name_binding_type },
            spans: smallvec![*local_value.name.span(), expr_span],
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn cycle_in_local_values(names: Vec<IdentWithSpan>) -> Self {
        MirError {
            kind: MirErrorKind::CycleInLocalValues { names: names.iter().map(|name| name.id()).collect() },
            spans: names.iter().map(|name| *name.span()).collect(),
            extra: ExtraErrorInfo::none(),
        }
    }

    pub fn missing_fields_in_struct_constructor(span: SpanRange, names: Vec<InternedString>, struct_name: InternedString) -> Self {
        MirError {
            kind: MirErrorKind::MissingFieldsInStructConstructor {
                names: names,
                struct_name,
            },
            spans: smallvec![span],
            extra: ExtraErrorInfo::none(),
        }
    }

    // NOTE: it's quite expensive because it searches for similar names
    pub fn unknown_fields_in_struct_constructor(
        unknown_names: Vec<IdentWithSpan>,
        valid_names: Vec<InternedString>,
        struct_name: InternedString,
    ) -> Self {
        let mut extra = ExtraErrorInfo::none();
        let valid_names_u8 = valid_names.iter().map(|name| unintern_string(*name)).collect::<Vec<_>>();

        for unknown_name in unknown_names.iter() {
            let unknown_name_u8 = unintern_string(unknown_name.id());

            match find_similar_names(
                &unknown_name_u8,
                &valid_names_u8,
            ) {
                names if !names.is_empty() => {
                    extra.push_message(format!(
                        "`{}`: Do you mean {}?",
                        String::from_utf8_lossy(&unknown_name_u8).to_string(),
                        concat_commas(
                            &names,
                            "or",
                            "`",
                            "`",
                        ),
                    ));
                },
                _ => {},
            }
        }

        MirError {
            kind: MirErrorKind::UnknownFieldsInStructConstructor {
                names: unknown_names.iter().map(|name| name.id()).collect(),
                struct_name,
            },
            spans: unknown_names.iter().map(|name| *name.span()).collect(),
            extra,
        }
    }

    pub fn not_a_struct(
        expr: &hir::Expr,
    ) -> Self {
        let rendered_expr = match expr.to_string() {
            e if e.len() < 16 => Some(e),
            _ => None,
        };

        MirError {
            kind: MirErrorKind::NotAStruct {
                rendered_expr,
            },
            spans: smallvec![expr.span],
            extra: ExtraErrorInfo::none(),
        }
    }
}

impl SodigyError<MirErrorKind> for MirError {
    fn get_mut_error_info(&mut self) -> &mut ExtraErrorInfo {
        &mut self.extra
    }

    fn get_error_info(&self) -> &ExtraErrorInfo {
        &self.extra
    }

    fn get_first_span(&self) -> Option<SpanRange> {
        self.spans.get(0).copied()
    }

    fn get_spans(&self) -> &[SpanRange] {
        &self.spans
    }

    fn error_kind(&self) -> &MirErrorKind {
        &self.kind
    }

    fn index(&self) -> u32 {
        8
    }

    fn get_stage(&self) -> Stage {
        Stage::Mir
    }
}

pub enum MirErrorKind {
    RecursiveLocalValue { name: InternedString, name_binding_type: NameBindingType },
    CycleInLocalValues { names: Vec<InternedString> },
    MissingFieldsInStructConstructor { names: Vec<InternedString>, struct_name: InternedString },
    UnknownFieldsInStructConstructor { names: Vec<InternedString>, struct_name: InternedString },
    NotAStruct { rendered_expr: Option<String> },
}

impl SodigyErrorKind for MirErrorKind {
    fn msg(&self, _: &mut InternSession) -> String {
        match self {
            MirErrorKind::RecursiveLocalValue { .. } => format!("a recursive local name binding"),
            MirErrorKind::CycleInLocalValues { names } => format!(
                "a cycle in local values: {}",
                concat_commas(
                    &names.iter().map(|name| name.to_string()).collect::<Vec<_>>(),
                    "and",
                    "`",
                    "`",
                ),
            ),
            MirErrorKind::MissingFieldsInStructConstructor { names, struct_name } => format!(
                "missing field{} {} in a struct constructor of `{struct_name}`",
                if names.len() > 1 { "s" } else { "" },
                concat_commas(
                    &names.iter().map(|name| name.to_string()).collect::<Vec<_>>(),
                    "and",
                    "`",
                    "`",
                ),
            ),
            MirErrorKind::UnknownFieldsInStructConstructor { names, struct_name } => format!(
                "no field named `{}` on struct `{struct_name}`",
                concat_commas(
                    &names.iter().map(|name| name.to_string()).collect::<Vec<_>>(),
                    "and",
                    "`",
                    "`",
                ),
            ),
            MirErrorKind::NotAStruct { .. } => String::from("invalid struct constructor"),
        }
    }

    fn help(&self, _: &mut InternSession) -> String {
        match self {
            MirErrorKind::RecursiveLocalValue { name, name_binding_type } => format!(
                "{} {} `{name}` is referencing it self.",
                name_binding_type.article(true),
                name_binding_type.render_error(),
            ),
            MirErrorKind::NotAStruct { rendered_expr } => if let Some(rendered_expr) = rendered_expr {
                format!("`{rendered_expr}` is not a struct-like object.")
            } else {
                String::from("It is not a struct-like object.")
            },
            MirErrorKind::CycleInLocalValues { .. }
            | MirErrorKind::MissingFieldsInStructConstructor { .. }
            | MirErrorKind::UnknownFieldsInStructConstructor { .. } => String::new(),
        }
    }

    fn index(&self) -> u32 {
        match self {
            MirErrorKind::RecursiveLocalValue { .. } => 0,
            MirErrorKind::CycleInLocalValues { .. } => 1,
            MirErrorKind::MissingFieldsInStructConstructor { .. } => 2,
            MirErrorKind::UnknownFieldsInStructConstructor { .. } => 3,
            MirErrorKind::NotAStruct { .. } => 4,
        }
    }
}

fn find_similar_names(
    name: &[u8],
    names: &[Vec<u8>],
) -> Vec<String> {
    let mut result = vec![];

    // distance("f", "x") = 1, but it's not a good suggestion
    // distance("foo", "goo") = 1, and it seems like a good suggestion
    // distance("f", "F") = 0, and it seems like a good suggestion
    let similarity_threshold = (name.len() / 3).max(1);

    for candidate in names.iter() {
        if substr_edit_distance(name, candidate) < similarity_threshold {
            result.push(String::from_utf8_lossy(candidate).to_string());
        }

        if result.len() > 3 {
            break;
        }
    }

    result
}
