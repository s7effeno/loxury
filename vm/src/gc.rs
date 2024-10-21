// TODO: implement Gc<T> and trait

use std::marker::PhantomData;
use std::hash::Hash;

use crate::chunk::{Chunk, Function, FunctionKind, Value};


#[derive(Debug)]
pub struct GcHandle<T> {
    idx: usize,
    marked: bool,
    _type: PhantomData<T>
}

impl<T> PartialEq for GcHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx
    }
}

impl<T: PartialEq> Eq for GcHandle<T> {
}

impl Hash for GcHandle<String> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.idx.hash(state)
    }
}

impl<T> Clone for GcHandle<T> {
    fn clone(&self) -> Self {
        Self::new(self.idx)
    }
}

impl<T> Copy for GcHandle<T> { }

trait Gc {
    fn new(v: Self) -> GcHandle<Self> where Self: Sized;
}

impl<T> GcHandle<T> {
    pub fn uninit() -> Self {
        // ugly but realistically it never reaches max
        Self {
            idx: usize::MAX,
            marked: false,
            _type: PhantomData,
        }
    }

    pub fn new(idx: usize) -> Self {
        Self {
            idx,
            marked: false,
            _type: PhantomData,
        }
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
    pub fn new_string(&mut self, s: String) -> GcHandle<String> {
        // FIXME: this interning truly sucks
        // without this global variable resolving doesnt work
        if let Some(v) = self.strings.iter().position(|x| x == &s) {
            GcHandle::new(v)
        } else {
            self.strings.push(s);
            GcHandle::new(self.strings.len() - 1)
        }
    }

    pub fn get_string(&self, s: GcHandle<String>) -> &str {
        &self.strings[s.idx]
    }

    pub fn new_function(&mut self, kind: FunctionKind) -> GcHandle<Function> {
        let function = Function::new(kind);
        self.functions.push(function);
        GcHandle::new(self.functions.len() - 1)
    }

    pub fn get_function(&mut self, f: GcHandle<Function>) -> &mut Function {
        &mut self.functions[f.idx]
    }

    // TODO: move to better place(?)
    pub fn print_value(&mut self, value: Value) {
        match value {
            Value::Bool(v) => print!("{v}"),
            Value::Nil => print!("nil"),
            Value::Number(v) => print!("{v}"),
            Value::String(v) => {
                let v = self.get_string(v);
                print!("{v}")
            }
            Value::Function(v) => {
                let v = self.get_function(v);
                print!("{v}")
            }
        }
    }
}
