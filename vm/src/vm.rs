use crate::{
    chunk::{Chunk, OpCode, Value},
    compiler::Compiler,
    location::AtCoords,
    RunError,
};
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

    fn error(&mut self, error: RunError) {
        eprintln!("{}", self.chunk.coords(self.ip).locate(error));
        self.had_error = true;
    }

    fn read_byte(&mut self) -> u8 {
        let ret = self.chunk.byte_at(self.ip);
        self.ip += 1;
        ret
    }

    pub fn run(&mut self) -> Result<(), ()> {
        loop {
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
                        _ => todo!(),
                    }
                }
                OpCode::Subtract => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.stack.push(Value::Number(a - b))
                        }
                        _ => todo!(),
                    }
                }
                OpCode::Multiply => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.stack.push(Value::Number(a * b))
                        }
                        _ => todo!(),
                    }
                }
                OpCode::Divide => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.stack.push(Value::Number(a / b))
                        }
                        _ => todo!(),
                    }
                }
                OpCode::Negate => match self.stack.pop() {
                    Value::Number(n) => self.stack.push(Value::Number(-n)),
                    _ => todo!(),
                },
            }
        }
    }
}
