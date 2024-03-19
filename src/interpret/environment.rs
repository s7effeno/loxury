use crate::error::Runtime as RuntimeError;
use crate::parse::Literal;
use crate::Located;
use crate::Object;
use std::collections::HashMap;
use std::mem;

pub struct Environment {
    values: HashMap<String, Object>,
    enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn nest(&mut self) {
        let enclosing = mem::replace(self, Self::new());
        self.enclosing = Some(Box::new(enclosing));
    }

    pub fn unnest(&mut self) -> Result<(), ()> {
        let enclosing = mem::take(&mut self.enclosing);
        if let Some(e) = enclosing {
            let _ = mem::replace(self, *e);
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn define(&mut self, name: &str, value: Object) {
        self.values.insert(name.to_owned(), value);
    }

    pub fn assign(&mut self, name: &str, value: Object) -> Result<(), ()> {
        match self.values.get_mut(name) {
            Some(v) => {
                *v = value;
                Ok(())
            }
            None => match &mut self.enclosing {
                Some(ref mut e) => e.assign(name, value),
                None => Err(()),
            },
        }
    }

    pub fn get(&self, name: &str) -> Result<&Object, ()> {
        match self.values.get(name) {
            Some(v) => Ok(v),
            None => match &self.enclosing {
                Some(e) => e.get(name),
                None => Err(()),
            },
        }
    }
}
