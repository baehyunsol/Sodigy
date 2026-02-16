use crate::Session;
use sodigy_mir::{Expr, Type, type_of};
use sodigy_parse::Field;
use sodigy_string::InternedString;

// Imagine `struct Person = { age: _, name: _ };` and `p = Person { .. };`.
// It lowers `p.age` to `p._0`.
// inter-mir did the type-check and there're no type errors.
pub(crate) fn lower_fields(lhs: &Expr, fields: &mut Vec<Field>, session: &mut Session) {
    let mut curr_type = type_of(lhs, &session.types, &session.struct_shapes, &session.lang_items).unwrap();
    let last_index = fields.len();

    for (i, field) in fields.iter_mut().enumerate() {
        match &curr_type {
            Type::Static { def_span, .. } => match field {
                Field::Name { name, .. } => {
                    let struct_shape = session.struct_shapes.get(def_span).unwrap();

                    for (j, field_def) in struct_shape.fields.iter().enumerate() {
                        if *name == field_def.name {
                            *field = Field::Index(j as i64);
                            break;
                        }
                    }
                },
                _ => {
                    // nothing to lower
                },
            },
            Type::Tuple { args, .. } => match field {
                Field::Name { name, .. } => {
                    for j in 0..args.len() {
                        // TODO: Why isn't `name.eq(format!("_{j}").as_bytes())` working? Is it rustc bug?
                        if InternedString::eq(name, format!("_{j}").as_bytes()) {
                            *field = Field::Index(j as i64);
                            break;
                        }
                    }
                },
                Field::Index(j) if *j < 0 => todo!(),
                _ => {
                    // nothing to lower
                },
            },
            _ => todo!(),
        }

        if i + 1 != last_index {
            curr_type = session.get_type_of_field(&curr_type, &[field.clone()]).unwrap();
        }
    }
}
