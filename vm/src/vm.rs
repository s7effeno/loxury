use crate::chunk::{Chunk, Function, FunctionKind, OpCode, Value};
use crate::compiler::Compiler;
use crate::location::AtCoords;
use crate::{ArrayVec, RunError};
use crate::gc::{GcHandle, Manager};
use std::collections::HashMap;

struct CallFrame {
    function: GcHandle<Function>,
    ip: usize,
    base: usize,
}

impl CallFrame {
    fn new(function: GcHandle<Function>, base: usize) -> Self {
        Self {
            function,
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
    objects: Manager,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            frames: ArrayVec::new(),
            stack: ArrayVec::new(),
            globals: HashMap::new(),
            objects: Manager::new(),
        }
    }

    pub fn run(&mut self, source: &str) -> Result<(), ()> {
        let function = Compiler::compile(source, &mut self.objects, FunctionKind::Script)?; 
        // self.function = function;
        // let function = self.objects.get_function(function);
        // let _ = function.chunk.disassemble(&self.objects);

        self.frames.push(CallFrame::new(function, 0));
        self.execute(function).map_err(|e| {
            println!("{e}");
            ()
        })
    }

    fn current_frame(&mut self) -> &mut CallFrame {
        self.frames.last_mut().unwrap()
    }

    fn error(&mut self, error: RunError) -> Result<(), AtCoords<RunError>> {
        /*let frame = self.current_frame();
        let ip = frame.ip;
        Err(frame.function.coords(ip).locate(error))*/
        todo!()
    }

    fn is_falsey(value: Value) -> bool {
        match value {
            Value::Nil => true,
            Value::Bool(b) => !b,
            _ => false,
        }
    }

    fn execute(&mut self, function: GcHandle<Function>) -> Result<(), AtCoords<RunError>> {
        loop {
            macro_rules! function {
                () => {
                    self.objects.get_function(function)
                };
            }
            macro_rules! read_byte {
                () => {{
                    let frame = self.frames.last_mut().unwrap();
                    let ret = function!().chunk.byte_at(frame.ip);
                    frame.ip += 1;
                    ret
                }}
            }
            macro_rules! read_wide {
                () => {
                    u16::from_be_bytes([read_byte!(), read_byte!()])
                }
            }
            macro_rules! read_constant {
                () => {
                    function!().chunk.get_constant(read_byte!())
                }
            }
            // TODO: add macro for binary expressions
            match read_byte!().try_into().unwrap() {
                OpCode::Return => {
                    return Ok(());
                }
                OpCode::Constant => {
                    let constant = read_constant!().clone();
                    self.stack.push(constant);
                }
                OpCode::Add => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.stack.push(Value::Number(a + b))
                        }
                        (Value::String(a), Value::String(b)) => {
                            let value = self.objects.get_string(a).to_owned() + self.objects.get_string(b);
                            let value = self.objects.new_string(value);
                            self
                                .stack
                                .push(Value::String(value));
                        }
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
                    let value = self.stack.last().unwrap().clone();
                    self.print_value(value);
                    println!("");
                    self.stack.pop();
                    // println!("{}", self.stack.pop());
                }
                OpCode::Pop => {
                    self.stack.pop();
                }
                OpCode::DefineGlobal => {
                    let name = read_constant!().try_as_string().unwrap();
                    let value = self.stack.pop();
                    self.globals.insert(name, value);
                }
                OpCode::GetGlobal => {
                    let name = read_constant!().try_as_string().unwrap();
                    if let Some(value) = self.globals.get(&name) {
                        self.stack.push(value.clone());
                    } else {
                        let name = self.objects.get_string(name).into();
                        return self.error(RunError::UndefinedVariable(name));
                    }
                }
                OpCode::SetGlobal => {
                    let name = read_constant!().try_as_string().unwrap();
                    if let Some(value) = self.globals.get_mut(&name) {
                        let new_value = self.stack.last().unwrap().clone();
                        *value = new_value;
                    } else {
                        let name = self.objects.get_string(name).into();
                        return self.error(RunError::UndefinedVariable(name));
                    }
                }
                OpCode::GetLocal => {
                    let slot = read_byte!();
                    let slot = self.current_frame().base + slot as usize;
                    let value = self.stack.get(slot as usize).unwrap().clone();
                    self.stack.push(value);
                }
                OpCode::SetLocal => {
                    let slot = read_byte!();
                    let slot = self.current_frame().base + slot as usize;
                    let value = self.stack.last().unwrap().clone();
                    *self.stack.get_mut(slot as usize).unwrap() = value;
                }
                OpCode::JumpIfFalse => {
                    let offset = u16::from_be_bytes([read_byte!(), read_byte!()]);
                    if Self::is_falsey(self.stack.last().unwrap().clone()) {
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
            }
        }
    }

    fn print_value(&mut self, value: Value) {
        self.objects.print_value(value);
    }
}
