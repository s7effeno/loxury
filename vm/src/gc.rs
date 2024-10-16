// TODO: implement Gc<T> and trait

use crate::chunk::{Function, Chunk};


#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Gc(usize);

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

    pub fn get_function(&mut self, f: Gc) -> &Function {
        &self.functions[f.0]
    }
}
