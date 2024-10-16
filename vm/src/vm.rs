use crate::chunk::{Chunk, OpCode, Value};
use crate::compiler::Compiler;
use crate::location::AtCoords;
use crate::RunError;
use crate::gc::{Gc, Manager};
use std::collections::HashMap;
use std::mem::MaybeUninit;

struct Stack {
    values: [MaybeUninit<Value>; u8::MAX as usize + 1],
    count: u8,
}

impl Stack {
    fn new() -> Self {
        Self {
            values: unsafe { MaybeUninit::uninit().assume_init() },
            count: 0,
        }
    }

    fn push(&mut self, value: Value) {
        assert_ne!(self.count, 255);
        (self.values[self.count as usize]).write(value);
        self.count += 1;
    }

    fn pop(&mut self) -> Value {
        assert_ne!(self.count, 0);
        self.count -= 1;
        unsafe { (self.values[self.count as usize]).assume_init_ref() }.clone()
    }

    fn peek(&self) -> Value {
        assert_ne!(self.count, 0);
        unsafe { (self.values[(self.count - 1) as usize]).assume_init_ref() }.clone()
    }

    fn get(&mut self, slot: u8) -> Value {
        assert!(slot < self.count);
        unsafe { self.values[slot as usize].assume_init_ref() }.clone()
    }

    fn set(&mut self, slot: u8, value: Value) {
        // assert!(slot < self.count);
        self.values[slot as usize].write(value);
    }
}

pub struct Vm {
    ip: usize,
    stack: Stack,
    // name -> value
    globals: HashMap<Gc, Value>,
    objects: Manager,
    function: Gc,
}

impl Vm {
    pub fn new(source: &str) -> Result<Self, ()> {
        // TODO: compile in a separate phase
        let mut objects = Manager::new();
        let function = Compiler::compile(source, &mut objects)?;
        function.asdf;
        Ok(Self {
            ip: 0,
            stack: Stack::new(),
            globals: HashMap::new(),
            objects: Manager::new(),
        })
    }

    fn error(&mut self, error: RunError) -> Result<(), AtCoords<RunError>> {
        Err(self.chunk.coords(self.ip).locate(error))
    }

    fn read_byte(&mut self) -> u8 {
        let ret = self.chunk.byte_at(self.ip);
        self.ip += 1;
        ret
    }

    fn read_constant(&mut self) -> Value {
        let byte = self.read_byte();
        self.chunk.get_constant(byte).clone()
    }

    fn is_falsey(value: Value) -> bool {
        match value {
            Value::Nil => true,
            Value::Bool(b) => !b,
            _ => false,
        }
    }

    pub fn run(&mut self) -> Result<(), AtCoords<RunError>> {
        loop {
            // TODO: add macro for binary expressions
            match self.read_byte().try_into().unwrap() {
                OpCode::Return => {
                    return Ok(());
                }
                OpCode::Constant => {
                    let constant = self.read_constant().clone();
                    self.stack.push(constant);
                }
                OpCode::Add => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.stack.push(Value::Number(a + b))
                        }
                        (Value::String(a), Value::String(b)) => self
                                .stack
                                .push(Value::String(self.objects.new_string(self.objects.get_string(a).to_owned() + self.objects.get_string(b)))),
                        _ => return self.error(RunError::ExpectedNumbersOrStrings),
                    }
                }
                OpCode::Subtract => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.stack.push(Value::Number(a - b))
                        }
                        _ => return self.error(RunError::ExpectedNumbers),
                    }
                }
                OpCode::Multiply => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.stack.push(Value::Number(a * b))
                        }
                        _ => return self.error(RunError::ExpectedNumbers),
                    }
                }
                OpCode::Divide => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.stack.push(Value::Number(a / b))
                        }
                        _ => return self.error(RunError::ExpectedNumbers),
                    }
                }
                OpCode::Negate => match self.stack.pop() {
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
                    let value = Self::is_falsey(self.stack.pop());
                    self.stack.push(Value::Bool(value));
                }
                OpCode::Equal => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    // TODO: implement separate function
                    self.stack.push(Value::Bool(match (a, b) {
                        (Value::Bool(a), Value::Bool(b)) => a == b,
                        (Value::Nil, Value::Nil) => true,
                        (Value::Number(a), Value::Number(b)) => a == b,
                        _ => false,
                    }));
                }
                OpCode::Greater => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => self.stack.push(Value::Bool(a > b)),
                        _ => return self.error(RunError::ExpectedNumbers),
                    }
                }
                OpCode::Less => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => self.stack.push(Value::Bool(a < b)),
                        _ => return self.error(RunError::ExpectedNumbers),
                    }
                }
                OpCode::Print => {
                    println!("{}", self.stack.pop());
                }
                OpCode::Pop => {
                    self.stack.pop();
                }
                OpCode::DefineGlobal => {
                    let name = self.read_constant().try_as_string().unwrap();
                    let value = self.stack.pop();
                    self.globals.insert(name, value);
                }
                OpCode::GetGlobal => {
                    let name = self.read_constant().try_as_string().unwrap();
                    if let Some(value) = self.globals.get(&name) {
                        self.stack.push(value.clone());
                    } else {
                        return self.error(RunError::UndefinedVariable(self.objects.get_string(name).into()));
                    }
                }
                OpCode::SetGlobal => {
                    let name = self.read_constant().try_as_string().unwrap();
                    if let Some(value) = self.globals.get_mut(&name) {
                        let new_value = self.stack.peek();
                        *value = new_value;
                    } else {
                        return self.error(RunError::UndefinedVariable(self.objects.get_string(name).into()));
                    }
                }
                OpCode::GetLocal => {
                    let slot = self.read_byte();
                    let value = self.stack.get(slot);
                    self.stack.push(value);
                }
                OpCode::SetLocal => {
                    let slot = self.read_byte();
                    let value = self.stack.peek();
                    self.stack.set(slot, value);
                }
                OpCode::JumpIfFalse => {
                    let offset = u16::from_be_bytes([self.read_byte(), self.read_byte()]);
                    if Self::is_falsey(self.stack.peek()) {
                        self.ip += offset as usize;
                    }
                }
                OpCode::Jump => {
                    let offset = u16::from_be_bytes([self.read_byte(), self.read_byte()]);
                    self.ip += offset as usize;
                }
                OpCode::Loop => {
                    let offset = u16::from_be_bytes([self.read_byte(), self.read_byte()]);
                    self.ip -= offset as usize;
                }
            }
        }
    }
}
