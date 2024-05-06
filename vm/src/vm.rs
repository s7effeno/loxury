use crate::chunk::{Chunk, OpCode, Value};
use crate::compiler::Compiler;
use crate::RunError;
use std::mem::MaybeUninit;

struct Stack {
    values: [MaybeUninit<Value>; 256],
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
        unsafe { (self.values[self.count as usize]).assume_init() }
    }
}

pub struct Vm {
    chunk: Chunk,
    ip: usize,
    stack: Stack,
    had_error: bool,
}

impl Vm {
    pub fn new(source: &str) -> Result<Self, ()> {
        let mut chunk = Chunk::new();
        Compiler::compile(source, &mut chunk)?;
        Ok(Self {
            chunk,
            ip: 0,
            stack: Stack::new(),
            had_error: false,
        })
    }

    fn error(&mut self, error: RunError) -> Result<(), ()> {
        eprintln!("{}", self.chunk.coords(self.ip).locate(error));
        self.had_error = true;
        Err(())
    }

    fn read_byte(&mut self) -> u8 {
        let ret = self.chunk.byte_at(self.ip);
        self.ip += 1;
        ret
    }

    pub fn run(&mut self) -> Result<(), ()> {
        loop {
            // TODO: add macro for binary expressions
            match self.read_byte().try_into().unwrap() {
                OpCode::Return => {
                    println!("{:?}", self.stack.pop());
                    return Ok(());
                }
                OpCode::Constant => {
                    let idx = self.read_byte();
                    let constant = self.chunk.get_constant(idx);
                    self.stack.push(constant);
                }
                OpCode::Add => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.stack.push(Value::Number(a + b))
                        }
                        _ => self.error(RunError::ExpectedNumbers)?,
                    }
                }
                OpCode::Subtract => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.stack.push(Value::Number(a - b))
                        }
                        _ => self.error(RunError::ExpectedNumbers)?,
                    }
                }
                OpCode::Multiply => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.stack.push(Value::Number(a * b))
                        }
                        _ => self.error(RunError::ExpectedNumbers)?,
                    }
                }
                OpCode::Divide => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.stack.push(Value::Number(a / b))
                        }
                        _ => self.error(RunError::ExpectedNumbers)?,
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
                    fn is_falsey(value: Value) -> bool {
                        match value {
                            Value::Nil => true,
                            Value::Bool(b) => !b,
                            _ => false,
                        }
                    }
                    let value = is_falsey(self.stack.pop());
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
                        _ => self.error(RunError::ExpectedNumbers)?,
                    }
                }
                OpCode::Less => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => self.stack.push(Value::Bool(a < b)),
                        _ => self.error(RunError::ExpectedNumbers)?,
                    }
                }
            }
        }
    }
}
