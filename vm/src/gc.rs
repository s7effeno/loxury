// TODO: implement Gc<T> and trait

use std::hash::Hash;
use std::marker::PhantomData;

use crate::chunk::{Closure, Function, FunctionKind, ObjUpvalue, Value};

#[derive(Debug)]
pub struct GcHandle<T> {
    idx: usize,
    marked: bool,
    _type: PhantomData<T>,
}

impl<T: Gc> GcHandle<T> {
    pub fn mark(&mut self,  manager: &mut Manager) {
        if !self.marked {
            self.marked = true;
            T::greyen(manager, self);
        }
    }
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

    fn get_mut(manager: &mut Manager, handle: GcHandle<Self>) -> &mut Self
    where
        Self: Sized;

    fn greyen(manager: &mut Manager, handle: GcHandle<Self>) where Self: Sized;

    fn blacken(manager: &mut Manager, handle: &mut GcHandle<Self>)
        where Self: Sized;
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
    strings_grey: Vec<GcHandle<String>>,
    functions: Vec<Function>,
    functions_grey: Vec<GcHandle<Function>>,
    closures: Vec<Closure>,
    closures_grey: Vec<GcHandle<Closure>>,
    upvalues: Vec<ObjUpvalue>,
    upvalues_grey: Vec<GcHandle<ObjUpvalue>>,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            strings: Vec::new(),
            strings_grey: Vec::new(),
            functions: Vec::new(),
            functions_grey: Vec::new(),
            closures: Vec::new(),
            closures_grey: Vec::new(),
            upvalues: Vec::new(),
            upvalues_grey: Vec::new(),
        }
    }

    pub fn add<T: Gc>(&mut self, value: T) -> GcHandle<T> {
        T::new(self, value)
    }

    pub fn get<T: Gc>(&self, handle: GcHandle<T>) -> &T {
        T::get(self, handle)
    }

    pub fn get_mut<T: Gc>(&mut self, handle: GcHandle<T>) -> &mut T {
        T::get_mut(self, handle)
    }

    pub fn mark<T: Gc>(&mut self, handle: &mut GcHandle<T>) {
        handle.mark(self)
    }

    pub fn mark_value(&mut self, value: Value) {
        match value {
            Value::Bool(_) => todo!(),
            Value::Nil => todo!(),
            Value::Number(_) => todo!(),
            Value::String(v) => v.mark(self),
            Value::Function(v) => todo!(),
            Value::NativeFunction { arity, f } => todo!(),
            Value::Closure(gc_handle) => todo!(),
        }
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

    fn get_mut(_manager: &mut Manager, _handle: GcHandle<Self>) -> &mut Self
    where
        Self: Sized,
    {
        unimplemented!()
    }

    fn greyen(manager: &mut Manager, handle: GcHandle<Self>) where Self: Sized {
        manager.strings_grey.push(handle);
    }

    fn blacken(_manager: &mut Manager, _handle: &mut GcHandle<Self>)
        where Self: Sized {
    }
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

    fn get_mut(manager: &mut Manager, handle: GcHandle<Self>) -> &mut Self
    where
        Self: Sized,
    {
        &mut manager.functions[handle.idx]
    }

    fn greyen(manager: &mut Manager, handle: GcHandle<Self>) where Self: Sized {
        manager.functions_grey.push(handle);
    }

    fn blacken(manager: &mut Manager, handle: &mut GcHandle<Self>)
        where Self: Sized {
            let f = manager.get(*handle);
            for constant in f.chunk.constants {

            }
    }
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

    fn get_mut(manager: &mut Manager, handle: GcHandle<Self>) -> &mut Self {
        &mut manager.closures[handle.idx]
    }

    fn greyen(manager: &mut Manager, handle: GcHandle<Self>) where Self: Sized {
        manager.closures_grey.push(handle)
    }

    fn blacken(manager: &mut Manager, handle: &mut GcHandle<Self>)
        where Self: Sized {
        todo!()
    }
}

impl Gc for ObjUpvalue {
    fn new(manager: &mut Manager, value: Self) -> GcHandle<Self>
    where
        Self: Sized {
            manager.upvalues.push(value);
            GcHandle::new(manager.upvalues.len() - 1)
    }

    fn get(manager: &Manager, handle: GcHandle<Self>) -> &Self
    where
        Self: Sized {
            &manager.upvalues[handle.idx]
    }

    fn get_mut(manager: &mut Manager, handle: GcHandle<Self>) -> &mut Self
    where
        Self: Sized {
            &mut manager.upvalues[handle.idx]
    }

    fn greyen(manager: &mut Manager, handle: GcHandle<Self>) where Self: Sized {
        manager.upvalues_grey.push(handle);
    }

    fn blacken(manager: &mut Manager, handle: &mut GcHandle<Self>)
        where Self: Sized {
        todo!()
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

    pub fn collect_garbage(&mut self) {

    }
}
