use crate::error::Runtime as RuntimeError;
use crate::parse::Literal;
use crate::Located;
use crate::Object;
use std::cell::RefCell;
use std::collections::HashMap;
use std::mem;
use std::rc::Rc;

struct _Environment {
    values: HashMap<String, Object>,
    enclosing: Option<Environment>,
}

impl _Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }
}

#[derive(Clone)]
pub struct Environment(Rc<RefCell<_Environment>>);

impl Environment {
    pub fn new() -> Self {
        Self(RefCell::new(_Environment::new()).into())
    }

    pub fn define(&self, name: &str, value: Object) {
        self.0.borrow_mut().values.insert(name.to_owned(), value);
    }

    pub fn assign(&self, name: &str, value: Object) -> Result<(), ()> {
        let mut env = self.0.borrow_mut();
        match env.values.get_mut(name) {
            Some(v) => {
                *v = value;
                Ok(())
            }
            None => match &env.enclosing {
                Some(e) => e.assign(name, value),
                None => Err(()),
            },
        }
    }

    pub fn get(&self, name: &str) -> Result<Object, ()> {
        let env = self.0.borrow();
        match env.values.get(name) {
            Some(v) => Ok(v.clone()),
            None => match &env.enclosing {
                Some(e) => e.get(name),
                None => Err(()),
            },
        }
    }

    pub fn nest(&self) -> Self {
        let enclosing = Some(Self(self.0.clone()));
        Self(RefCell::new(_Environment {
            values: HashMap::new(),
            enclosing,
        }).into())
    }
}
