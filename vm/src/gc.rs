// TODO: implement Gc<T> and trait

use std::hash::Hash;
use std::marker::PhantomData;

use crate::chunk::{Closure, Function, FunctionKind, Value};

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

    /*fn get_mut(manager: &mut Manager, handle: GcHandle<Self>) -> &mut Self
    where
        Self: Sized;*/
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
    closures: Vec<Closure>,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            strings: Vec::new(),
            functions: Vec::new(),
            closures: Vec::new(),
        }
    }

    pub fn add<T: Gc>(&mut self, value: T) -> GcHandle<T> {
        T::new(self, value)
    }

    pub fn get<T: Gc>(&self, handle: GcHandle<T>) -> &T {
        T::get(self, handle)
    }

    /*pub fn get_mut<T: Gc>(&mut self, handle: GcHandle<T>) -> &mut T {
        T::get_mut(self, handle)
    }*/
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

    /*fn get_mut(_manager: &mut Manager, _handle: GcHandle<Self>) -> &mut Self
    where
        Self: Sized {
            unimplemented!()
    }*/
}

impl Gc for Function {
    fn new(manager: &mut Manager, value: Self) -> GcHandle<Self>
    where
        Self: Sized,
    {
        manager.functions.push(value);
        GcHandle::new(manager.functions.len() - 1)
    }

    fn get(manager: &Manager, handle: GcHandle<Self>) -> &Self
    where
        Self: Sized,
    {
        &manager.functions[handle.idx]
    }

    /*fn get_mut(manager: &mut Manager, handle: GcHandle<Self>) -> &mut Self
    where
        Self: Sized {
            &mut manager.functions[handle.idx]
    }*/
}

impl Gc for Closure {
    fn new(manager: &mut Manager, value: Self) -> GcHandle<Self>
    where
        Self: Sized,
    {
        manager.closures.push(value);
        GcHandle::new(manager.closures.len() - 1)
    }

    fn get(manager: &Manager, handle: GcHandle<Self>) -> &Self
    where
        Self: Sized,
    {
        &manager.closures[handle.idx]
    }
}

impl Manager {
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
                let v = self.get(v);
                print!("{v}")
            }
            Value::NativeFunction { .. } => {
                print!("<native fn>")
            }
            Value::Closure(v) => {
                let function = self.get(v).function;
                let function = self.get(function);
                print!("{function}");
            }
        }
    }
}
