use crate::error::Runtime as RuntimeError;
use crate::parse::Literal;
use crate::Located;
use std::collections::HashMap;

pub struct Environment {
    values: HashMap<String, Literal>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: String, value: Literal) -> Result<(), ()> {
        self.values.insert(name, value).ok_or(()).err().ok_or(())
    }

    pub fn get(&self, name: Located<String>) -> Result<Literal, Located<RuntimeError>> {
        self.values
            .get(name.value())
            .map(|l| Ok(l.clone()))
            .unwrap_or_else(|| {
                Err(name.co_locate(RuntimeError::UndefinedVariable(name.value().clone())))
            })
    }
}
