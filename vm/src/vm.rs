use crate::chunk::{Chunk, Value, OpCode};
use std::mem::MaybeUninit;

struct Stack {
    values: [MaybeUninit<Value>; 256],
    count: u8,
}

impl Stack {
    fn new() -> Self {
        Self {
            values: unsafe {
                MaybeUninit::uninit().assume_init()
            },
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

struct Vm {
    chunk: Chunk,
    ip: usize,
    stack: Stack,
}

impl Vm {
    fn read_byte(&mut self) -> u8 {
        let ret = self.chunk.byte_at(self.ip);
        self.ip += 1;
        ret
    }

    fn run(&mut self) -> Result<(), ()> {
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
                        (Value::Number(a), Value::Number(b)) => self.stack.push(Value::Number(a + b))
                    }
                }
                OpCode::Subtract => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => self.stack.push(Value::Number(a - b))
                    }
                }
                OpCode::Multiply => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => self.stack.push(Value::Number(a * b))
                    }
                }
                OpCode::Divide => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => self.stack.push(Value::Number(a / b))
                    }
                }
                OpCode::Negate => {
                    match self.stack.pop() {
                        Value::Number(n) => self.stack.push(Value::Number(-n))
                    }
                }
            }
        }
    }
}
