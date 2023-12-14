use crate::error::Runtime as RuntimeError;
use crate::parse::Literal;
use crate::Located;
use std::collections::HashMap;

struct Environment {
    values: HashMap<String, Literal>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    fn get(&mut self, name: Located<String>) -> Result<Literal, RuntimeError> {
        self.values
            .get(name.value())
            .map(|l| Ok(l.clone()))
            .unwrap_or_else(|| Err(RuntimeError::UndefinedVariable(name.value().clone())))
    }
}
