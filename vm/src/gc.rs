// TODO: implement Gc<T> and trait

use std::hash::Hash;
use std::marker::PhantomData;

use crate::chunk::{Function, FunctionKind, Value};

#[derive(Debug)]
pub struct GcHandle<T> {
    idx: usize,
    marked: bool,
    _type: PhantomData<T>,
}

impl<T> PartialEq for GcHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx
    }
}

impl<T: PartialEq> Eq for GcHandle<T> {}

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

impl<T> Copy for GcHandle<T> {}

pub trait Gc {
    fn new(manager: &mut Manager, value: Self) -> GcHandle<Self>
    where
        Self: Sized;

    fn get(manager: &Manager, handle: GcHandle<Self>) -> &Self
    where
        Self: Sized;
}

impl<T> GcHandle<T> {
    fn new(idx: usize) -> Self {
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

    pub fn add<T: Gc>(&mut self, value: T) -> GcHandle<T> {
        T::new(self, value)
    }

    pub fn get<T: Gc>(&self, handle: GcHandle<T>) -> &T {
        T::get(self, handle)
    }
}

impl Gc for String {
    fn new(manager: &mut Manager, value: Self) -> GcHandle<Self>
    where
        Self: Sized,
    {
        if let Some(v) = manager.strings.iter().position(|x| x == &value) {
            GcHandle::new(v)
        } else {
            manager.strings.push(value);
            GcHandle::new(manager.strings.len() - 1)
        }
    }

    fn get(manager: &Manager, handle: GcHandle<Self>) -> &Self
    where
        Self: Sized,
    {
        &manager.strings[handle.idx]
    }
}

impl Manager {
    pub fn new_function(&mut self, function: Function) -> GcHandle<Function> {
        self.functions.push(function);
        GcHandle::new(self.functions.len() - 1)
    }

    pub fn get_function_mut(&mut self, f: GcHandle<Function>) -> &mut Function {
        unsafe { self.functions.get_unchecked_mut(f.idx) }
    }

    pub fn get_function(&self, f: GcHandle<Function>) -> &Function {
        unsafe { self.functions.get_unchecked(f.idx) }
    }

    // TODO: move to better place(?)
    pub fn print_value(&self, value: Value) {
        match value {
            Value::Bool(v) => print!("{v}"),
            Value::Nil => print!("nil"),
            Value::Number(v) => print!("{v}"),
            Value::String(v) => {
                let v = self.get(v);
                print!("{v}")
            }
            Value::Function(v) => {
                let v = self.get_function(v);
                print!("{v}")
            }
            Value::NativeFunction { .. } => {
                print!("<native fn>")
            }
        }
    }
}
