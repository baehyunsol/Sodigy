use super::AST;

impl AST {

    // 1. If there's a lambda function, it creates a new function named `__LAMBDA_XXXX`.
    //   - All the uses of the lambda are replaced by `__LAMBDA_XXXX`.
    // 2. If there's a closure... how do I implement closures?
    pub fn resolve_lambdas(&mut self) {
        let mut new_funcs = vec![];

        for func in self.defs.values_mut() {

            for lambda in func.extract_lambdas() {
                new_funcs.push(lambda);
            }

        }

        for new_func in new_funcs.into_iter() {

            if let Some(_) = self.defs.insert(new_func.name, new_func) {
                // TODO: what do we do when there's a key collision?
                panic!("Internal Compiler Error 7AE7B0A");
            }

        }

    }
}
