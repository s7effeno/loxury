use std::marker::PhantomData;
use std::ops::{Index, IndexMut};
use std::{collections::HashMap, mem};

use crate::chunk::{Closure, Function, ObjUpvalue};

#[derive(Default)]
struct Interner {
    map: HashMap<&'static str, u32>,
    vec: Vec<&'static str>,
    buf: String,
    full: Vec<String>,
}

impl Interner {
    pub fn with_capacity(cap: usize) -> Interner {
        let cap = cap.next_power_of_two();
        Interner {
            map: HashMap::default(),
            vec: Vec::new(),
            buf: String::with_capacity(cap),
            full: Vec::new(),
        }
    }
    pub fn intern(&mut self, name: &str) -> u32 {
        if let Some(&id) = self.map.get(name) {
            return id;
        }
        let name = unsafe { self.alloc(name) };
        let id = self.map.len() as u32;
        self.map.insert(name, id);
        self.vec.push(name);
        debug_assert!(self.lookup(id) == name);
        debug_assert!(self.intern(name) == id);
        id
    }
    pub fn lookup(&self, id: u32) -> &str {
        self.vec[id as usize]
    }

    unsafe fn alloc(&mut self, name: &str) -> &'static str {
        let cap = self.buf.capacity();
        if cap < self.buf.len() + name.len() {
            let new_cap = (cap.max(name.len()) + 1).next_power_of_two();
            let new_buf = String::with_capacity(new_cap);
            let old_buf = mem::replace(&mut self.buf, new_buf);
            self.full.push(old_buf);
        }
        let interned = {
            let start = self.buf.len();
            self.buf.push_str(name);
            &self.buf[start..]
        };
        &*(interned as *const str)
    }
}

trait Trace<T: FnMut(GcHandle<Self>)>: Sized {
    fn trace(&self, tracer: T);
}

pub struct Arena<T> {
    objects: Vec<GcObject<T>>,
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self {
            objects: Vec::default()
        }
    }
}

struct GcObject<T> {
    value: T,
    marked: bool,
}

#[derive(Eq, Hash, PartialEq, Debug)]
pub struct GcHandle<T> {
    idx: usize,
    _type: PhantomData<*mut T>,
}

impl<T> GcHandle<T> {
    fn new(index: usize) -> Self {
        Self {
            idx: index,
            _type: PhantomData::default(),
        }
    }
}

impl<T> Clone for GcHandle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for GcHandle<T> {}

pub trait Allocate<T> {
    fn alloc(&mut self, value: T) -> GcHandle<T>;
}

macro_rules! define_heap {
    ($name:ident { $($arena:ident: $ty:ty),* $(,)? }) => {
        #[derive(Default)]
        pub struct $name {
            $($arena: $ty,)*
        }

        $(
            impl Allocate<<$ty as _Allocate>::Item> for $name {

                fn alloc(&mut self, value: <$ty as _Allocate>::Item) -> GcHandle<<$ty as _Allocate>::Item> {
                    self.$arena.alloc(value)
                }
            }

            impl Index<GcHandle<<$ty as _Allocate>::Item>> for $name {
                type Output = <$ty as Index<GcHandle< <$ty as _Allocate>::Item >>>::Output;

                fn index(&self, idx: GcHandle<<$ty as _Allocate>::Item>) -> &Self::Output {
                    &self.$arena[idx]
                }
            }

            impl IndexMut<GcHandle<<$ty as _Allocate>::Item>> for $name {
                fn index_mut(&mut self, idx: GcHandle<<$ty as _Allocate>::Item>) -> &mut Self::Output {
                    &mut self.$arena[idx]
                }
            }
        )*
    };
}

define_heap!(Heap {
    arena_function: Arena<Function>,
    arena_upvalues: Arena<ObjUpvalue>,
    arena_closure: Arena<Closure>,
    arena_string: StringArena,
});

#[derive(Default)]
pub struct StringArena {
    interner: Interner,
    arena: Arena<u32>,
}

impl _Allocate for StringArena {
    type Item = String;
    fn alloc(&mut self, value: Self::Item) -> GcHandle<Self::Item> {
        let index = self.interner.intern(&value);
        GcHandle::new(index as usize)
    }
}

impl Index<GcHandle<String>> for StringArena {
    type Output = str;
    fn index(&self, index: GcHandle<String>) -> &Self::Output {
        let index = self.arena[GcHandle::new(index.idx)];
        self.interner.lookup(index)
    }
}

impl IndexMut<GcHandle<String>> for StringArena {
    fn index_mut(&mut self, index: GcHandle<String>) -> &mut Self::Output {
        panic!()
    }
}

pub trait _Allocate {
    type Item;
    fn alloc(&mut self, value: Self::Item) -> GcHandle<Self::Item>;
}

impl<T> Index<GcHandle<T>> for Arena<T> {
    type Output = T;
    fn index(&self, idx: GcHandle<T>) -> &Self::Output {
        &self.objects[idx.idx].value
    }
}

impl<T> IndexMut<GcHandle<T>> for Arena<T> {
    fn index_mut(&mut self, idx: GcHandle<T>) -> &mut Self::Output {
        &mut self.objects[idx.idx].value
    }
}

impl<T> _Allocate for Arena<T> {
    type Item = T;
    fn alloc(&mut self, value: T) -> GcHandle<T> {
        self.objects.push(GcObject {
            value: value,
            marked: false,
        });
        GcHandle::new(self.objects.len() - 1)
    }
}

fn main() {}
