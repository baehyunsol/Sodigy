use super::check_call_args;
use crate::Expr;
use sodigy_error::Error;

impl Expr {
    pub fn check(&self) -> Result<(), Vec<Error>> {
        match self {
            Expr::Identifier { .. } |
            Expr::Number { .. } |
            Expr::String { .. } => Ok(()),
            Expr::If(r#if) => r#if.check(),
            Expr::Block(block) => block.check(),
            Expr::Call { func, args } => {
                let mut errors = vec![];

                if let Err(e) = func.check() {
                    errors.extend(e);
                }

                if let Err(e) = check_call_args(args) {
                    errors.extend(e);
                }

                if errors.is_empty() {
                    Ok(())
                }

                else {
                    Err(errors)
                }
            },
            Expr::StructInit {
                r#struct,
                fields,
            } => {
                let mut errors = vec![];

                if let Err(e) = r#struct.check() {
                    errors.extend(e);
                }

                for field in fields.iter() {
                    if let Err(e) = field.value.check() {
                        errors.extend(e);
                    }
                }

                if errors.is_empty() {
                    Ok(())
                }

                else {
                    Err(errors)
                }
            },
            Expr::Path { lhs, .. } => {
                let mut errors = vec![];

                if let Err(e) = lhs.check() {
                    errors.extend(e);
                }

                if errors.is_empty() {
                    Ok(())
                }

                else {
                    Err(errors)
                }
            },
            Expr::InfixOp { lhs, rhs, .. } |
            Expr::FieldModifier { lhs, rhs, .. } => {
                let mut errors = vec![];

                if let Err(e) = lhs.check() {
                    errors.extend(e);
                }

                if let Err(e) = rhs.check() {
                    errors.extend(e);
                }

                if errors.is_empty() {
                    Ok(())
                }

                else {
                    Err(errors)
                }
            },
            _ => panic!("TODO: {self:?}"),
        }
    }
}
