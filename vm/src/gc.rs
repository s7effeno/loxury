// TODO: implement Gc<T> and trait

use std::fmt;
use crate::chunk::{Function, Chunk, Value};


#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Gc(usize);

impl Gc {
    pub fn uninit() -> Self {
        Self(0)
    }
}

// FIXME: use generics
pub struct Manager {
    strings: Vec<String>,
    functions: Vec<Function>,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            strings: Vec::new(),
            functions: Vec::new(),
        }
    }
}

impl Manager {
    pub fn new_string(&mut self, s: String) -> Gc {
        self.strings.push(s);
        Gc(self.strings.len() - 1)
    }

    pub fn get_string(&mut self, s: Gc) -> &str {
        &self.strings[s.0]
    }

    pub fn new_function(&mut self) -> Gc {
        let function = Function {
            arity: 0,
            name: Gc(0),
            chunk: Chunk::new(),
        };
        self.functions.push(function);
        Gc(self.functions.len() - 1)
    }

    pub fn get_function(&mut self, f: Gc) -> &mut Function {
        &mut self.functions[f.0]
    }

    // TODO: move to better place(?)
    pub fn display(&mut self, value: Value, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match value {
            Value::Bool(v) => write!(f, "{v}"),
            Value::Nil => write!(f, "nil"),
            Value::Number(v) => write!(f, "{v}"),
            Value::String(v) => {
                let v = self.get_string(v);
                write!(f, "{v}")
            }
            Value::Function(v) => {
                let v = self.get_function(v);

            }
        }
    }
}
