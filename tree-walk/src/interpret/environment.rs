use super::Object;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
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

#[derive(Clone, Debug)]
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

    pub fn assign_at(&self, distance: usize, name: &str, value: Object) {
        self.ancestor(distance)
            .0
            .borrow_mut()
            .values
            .insert(name.to_string(), value);
    }

    pub fn get_at(&self, distance: usize, name: &str) -> Object {
        self.ancestor(distance)
            .0
            .borrow()
            .values
            .get(name)
            .unwrap()
            .clone()
    }

    pub fn ancestor(&self, distance: usize) -> Environment {
        if distance == 0 {
            self.clone()
        } else {
            self.0
                .borrow()
                .enclosing
                .as_ref()
                .unwrap()
                .ancestor(distance - 1)
        }
    }

    pub fn nest(&self) -> Self {
        let enclosing = Some(Self(self.0.clone()));
        Self(
            RefCell::new(_Environment {
                values: HashMap::new(),
                enclosing,
            })
            .into(),
        )
    }

    pub fn enclosing(&self) -> Option<Self> {
        self.0.borrow().enclosing.clone()
    }
}
