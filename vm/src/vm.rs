// TODO (speedup): clone and push the function instead of accessing it every time through the objects manager
// TODO: use infallible for `error`
use crate::chunk::{Closure, FunctionKind, ObjUpvalue, OpCode, Value, ValueDisplay};
use crate::compiler::Compiler;
use crate::gc::{Allocate, GcHandle, Heap, Mark, Trace};
use crate::lex::Lexer;
use crate::location::AtCoords;
use crate::{ArrayVec, RunError};
use std::collections::HashMap;

use std::time::UNIX_EPOCH;

struct CallFrame {
    closure: GcHandle<Closure>,
    ip: usize,
    base: usize,
}

impl CallFrame {
    fn new(closure: GcHandle<Closure>, base: usize) -> Self {
        Self {
            closure,
            base,
            ip: 0,
        }
    }
}

pub struct Vm {
    frames: ArrayVec<CallFrame, 64>,
    stack: ArrayVec<Value, { 64 * 256 }>,
    // name -> value
    globals: HashMap<GcHandle<String>, Value>,
    objects: Heap,
    open_upvalues: Vec<GcHandle<ObjUpvalue>>,
}

impl Vm {
    pub fn new() -> Self {
        let mut ret = Self {
            frames: ArrayVec::new(),
            stack: ArrayVec::new(),
            globals: HashMap::new(),
            objects: Heap::default(),
            open_upvalues: Vec::new(),
        };
        ret.define_native("clock", 0, |_| {
            Value::Number(UNIX_EPOCH.elapsed().unwrap().as_millis() as f64)
        });
        ret
    }

    pub fn define_native(&mut self, name: &str, arity: u8, f: fn(&[Value]) -> Value) {
        let name = self.objects.alloc(name.to_owned());
        self.stack.push(Value::String(name));
        self.globals
            .insert(name, Value::NativeFunction { arity, f });
        self.stack.pop();
    }

    pub fn run(&mut self, source: &str) -> Result<(), ()> {
        // clear previous junk
        // FIXME: check if needs optimization
        self.stack = ArrayVec::new();
        self.frames = ArrayVec::new();

        let function = Compiler::new(
            &mut Lexer::new(source).peekable(),
            &mut self.objects,
            FunctionKind::Script,
        )
        .compile()?;
        let function = self.objects.alloc(function);
        let closure = self.objects.alloc(Closure::new(function));

        self.frames.push(CallFrame::new(closure, 0));
        self.stack.push(Value::Closure(closure));
        self.execute().map_err(|e| {
            println!("{e}");
            ()
        })
    }

    fn current_frame(&mut self) -> &mut CallFrame {
        self.frames.last_mut().unwrap()
    }

    fn error(&mut self, error: RunError) -> Result<(), AtCoords<RunError>> {
        let ip = self.current_frame().ip;
        let closure = self.current_frame().closure;
        let function = &self.objects[closure].function;
        let function = &self.objects[*function];
        Err(function.chunk.coords(ip).locate(error))
    }

    fn is_falsey(value: Value) -> bool {
        match value {
            Value::Nil => true,
            Value::Bool(b) => !b,
            _ => false,
        }
    }

    fn execute(&mut self) -> Result<(), AtCoords<RunError>> {
        loop {
            macro_rules! read_byte {
                () => {{
                    let frame = self.frames.last_mut().unwrap();
                    let closure = &self.objects[frame.closure];
                    let function = &self.objects[closure.function];
                    let ip = frame.ip;
                    let ret = function.chunk.byte_at(ip);
                    frame.ip += 1;
                    ret
                }};
            }
            macro_rules! read_wide {
                () => {
                    u16::from_be_bytes([read_byte!(), read_byte!()])
                };
            }
            macro_rules! read_constant {
                () => {{
                    let frame = self.frames.last_mut().unwrap();
                    let closure = &self.objects[frame.closure];
                    let function = &self.objects[closure.function];
                    // let index = read_byte!();
                    function.chunk.get_constant(read_byte!())
                }};
            }
            // TODO: add macro for binary expressions
            match read_byte!().try_into().unwrap() {
                OpCode::Return => {
                    let result = self.stack.pop().unwrap();
                    let frame = self.frames.pop().unwrap();
                    self.close_upvalues(frame.base);
                    if self.frames.len() == 0 {
                        self.stack.pop();
                        return Ok(());
                    }

                    for _ in frame.base..self.stack.len() {
                        self.stack.pop();
                    }
                    self.stack.push(result);
                }
                OpCode::Constant => {
                    let constant = read_constant!();
                    self.stack.push(*constant);
                }
                OpCode::Add => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.stack.push(Value::Number(a + b))
                        }
                        (Value::String(a), Value::String(b)) => {
                            let value = self.objects[a].to_owned() + &self.objects[b];
                            let value = self.objects.alloc(value);
                            self.stack.push(Value::String(value));
                        }
                        _ => return self.error(RunError::ExpectedNumbersOrStrings),
                    }
                }
                OpCode::Subtract => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.stack.push(Value::Number(a - b))
                        }
                        _ => return self.error(RunError::ExpectedNumbers),
                    }
                }
                OpCode::Multiply => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.stack.push(Value::Number(a * b))
                        }
                        _ => return self.error(RunError::ExpectedNumbers),
                    }
                }
                OpCode::Divide => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.stack.push(Value::Number(a / b))
                        }
                        _ => return self.error(RunError::ExpectedNumbers),
                    }
                }
                OpCode::Negate => match self.stack.pop().unwrap() {
                    Value::Number(n) => self.stack.push(Value::Number(-n)),
                    _ => self.error(RunError::ExpectedNumber)?,
                },
                OpCode::Nil => {
                    self.stack.push(Value::Nil);
                }
                OpCode::True => {
                    self.stack.push(Value::Bool(true));
                }
                OpCode::False => {
                    self.stack.push(Value::Bool(false));
                }
                OpCode::Not => {
                    let value = Self::is_falsey(self.stack.pop().unwrap());
                    self.stack.push(Value::Bool(value));
                }
                OpCode::Equal => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    // TODO: implement separate function
                    self.stack.push(Value::Bool(match (a, b) {
                        (Value::Bool(a), Value::Bool(b)) => a == b,
                        (Value::Nil, Value::Nil) => true,
                        (Value::Number(a), Value::Number(b)) => a == b,
                        _ => false,
                    }));
                }
                OpCode::Greater => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => self.stack.push(Value::Bool(a > b)),
                        _ => return self.error(RunError::ExpectedNumbers),
                    }
                }
                OpCode::Less => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => self.stack.push(Value::Bool(a < b)),
                        _ => return self.error(RunError::ExpectedNumbers),
                    }
                }
                OpCode::Print => {
                    let value = self.stack.last().unwrap();
                    println!("{}", ValueDisplay(&value, &self.objects));
                    self.stack.pop();
                }
                OpCode::Pop => {
                    self.stack.pop();
                }
                OpCode::DefineGlobal => {
                    let name = read_constant!().try_as_string().unwrap();
                    let value = self.stack.pop().unwrap();
                    self.globals.insert(name, value);
                }
                OpCode::GetGlobal => {
                    let name = read_constant!().try_as_string().unwrap();
                    if let Some(value) = self.globals.get(&name) {
                        self.stack.push(*value);
                    } else {
                        let name = self.objects[name].into();
                        return self.error(RunError::UndefinedVariable(name));
                    }
                }
                OpCode::SetGlobal => {
                    let name = read_constant!().try_as_string().unwrap();
                    if let Some(value) = self.globals.get_mut(&name) {
                        let new_value = self.stack.last().unwrap();
                        *value = *new_value;
                    } else {
                        let name = self.objects[name].into();
                        return self.error(RunError::UndefinedVariable(name));
                    }
                }
                OpCode::GetLocal => {
                    let slot = read_byte!();
                    let slot = self.current_frame().base + slot as usize;
                    let value = self.stack[slot as usize];
                    self.stack.push(value);
                }
                OpCode::SetLocal => {
                    let slot = read_byte!();
                    let slot = self.current_frame().base + slot as usize;
                    let value = self.stack.last().unwrap();
                    self.stack[slot as usize] = *value;
                }
                OpCode::JumpIfFalse => {
                    let offset = u16::from_be_bytes([read_byte!(), read_byte!()]);
                    if Self::is_falsey(*self.stack.last().unwrap()) {
                        self.frames.last_mut().unwrap().ip += offset as usize;
                    }
                }
                OpCode::Jump => {
                    let offset = read_wide!();
                    self.current_frame().ip += offset as usize;
                }
                OpCode::Loop => {
                    let offset = read_wide!();
                    self.current_frame().ip -= offset as usize;
                }
                OpCode::Call => {
                    let args_count = read_byte!();
                    let base = self.stack.len() - 1 - args_count as usize;
                    match self.stack[base] {
                        Value::NativeFunction { arity, f } => {
                            if arity != args_count {
                                self.error(RunError::WrongArity(arity, args_count))?;
                            }
                            let result = f(&self.stack[base + 1..]);
                            // TODO: use `Vec::truncate`
                            for _ in 0..args_count + 1 {
                                self.stack.pop();
                            }
                            self.stack.push(result);
                        }
                        Value::Closure(c) => {
                            let closure = &self.objects[c];
                            let arity = self.objects[closure.function].arity;
                            if arity != args_count {
                                self.error(RunError::WrongArity(arity, args_count))?;
                            }
                            self.frames.push(CallFrame::new(c, base));
                        }
                        _ => self.error(RunError::NotCallable)?,
                    }
                }
                OpCode::Closure => {
                    let function = read_constant!().try_as_function().unwrap();
                    let closure = Closure::new(function);
                    let closure_obj = self.objects.alloc(closure);
                    self.stack.push(Value::Closure(closure_obj));

                    let closure = &self.objects[closure_obj];
                    let upvalue_count = self.objects[closure.function].upvalue_count;
                    for i in 0..upvalue_count {
                        let is_local = read_byte!();
                        let index = read_byte!();
                        if is_local == 1 {
                            let slot = self.current_frame().base + index as usize;
                            let upvalue = self.capture_upvalue(slot);
                            let closure = &mut self.objects[closure_obj];
                            closure.upvalues.push(upvalue);
                        } else {
                            let current_closure_obj = self.current_frame().closure;
                            let current_closure = &self.objects[current_closure_obj];
                            let upvalue = current_closure.upvalues[i].clone();
                            let closure = &mut self.objects[closure_obj];
                            closure.upvalues.push(upvalue);
                        }
                    }
                }
                OpCode::GetUpvalue => {
                    let slot = read_byte!();
                    let closure = self.current_frame().closure;
                    let closure = &self.objects[closure];
                    let value = self.get_upvalue(closure.upvalues[slot as usize]);
                    self.stack.push(value);
                }
                OpCode::SetUpvalue => {
                    let slot = read_byte!();
                    let closure = self.current_frame().closure;
                    let closure = &self.objects[closure];
                    let upvalue = closure.upvalues[slot as usize];
                    self.set_upvalue(upvalue);
                }
                OpCode::CloseUpvalue => {
                    let top = self.stack.len() - 1;
                    self.close_upvalues(top);
                    self.stack.pop();
                }
            }
        }
    }

    fn close_upvalues(&mut self, last: usize) {
        let mut it = self.open_upvalues.iter().rev();
        // FIXME: ugly
        let mut top = self.open_upvalues.len();
        while let Some(upvalue) = it.next() {
            let upvalue = &mut self.objects[*upvalue];
            let slot = upvalue.as_open().unwrap();
            if slot < last {
                break;
            }
            let value = self.stack[slot];
            *upvalue = ObjUpvalue::Closed(value);
            top -= 1;
        }
        self.open_upvalues.truncate(top);
    }

    // FIXME: ugly
    fn capture_upvalue(&mut self, slot: usize) -> GcHandle<ObjUpvalue> {
        let mut it = self.open_upvalues.iter().enumerate().rev();
        while let Some((i, upvalue_obj)) = it.next() {
            let upvalue = &self.objects[*upvalue_obj].as_open().unwrap();
            if *upvalue == slot {
                return *upvalue_obj;
            } else if *upvalue < slot {
                let upvalue = ObjUpvalue::Open(slot);
                let upvalue = self.objects.alloc(upvalue);
                self.open_upvalues.insert(i + 1, upvalue);
                return upvalue;
            }
        }
        let upvalue = ObjUpvalue::Open(slot);
        let upvalue = self.objects.alloc(upvalue);
        self.open_upvalues.push(upvalue);
        upvalue
    }

    fn get_upvalue(&self, upvalue: GcHandle<ObjUpvalue>) -> Value {
        let upvalue = &self.objects[upvalue];
        match upvalue {
            ObjUpvalue::Open(slot) => self.stack[*slot],
            ObjUpvalue::Closed(value) => *value,
        }
    }

    fn set_upvalue(&mut self, upvalue: GcHandle<ObjUpvalue>) {
        let upvalue = &mut self.objects[upvalue];
        let update = self.stack.last().unwrap();
        match upvalue {
            ObjUpvalue::Open(slot) => self.stack[*slot] = *update,
            ObjUpvalue::Closed(ref mut value) => *value = *update
        }
    }

    fn mark_roots(&self) {
        for slot in &*self.stack {
            slot.trace(&self.objects);
        }
        for frame in &*self.frames {
            let _ = self.objects.mark(frame.closure);
        }
        for upvalue in &*self.open_upvalues {
            self.objects.mark(*upvalue);
        }
        for (k, v) in &self.globals {
            self.objects.mark(*k);
            v.trace(&self.objects);
        }

        // don't care about compiler's temporary object, only collect at runtime
    }
}
