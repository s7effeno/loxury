use crate::chunk::{Function, FunctionKind, OpCode, Value};
use crate::compiler::Compiler;
use crate::gc::{GcHandle, Manager};
use crate::lex::Lexer;
use crate::location::AtCoords;
use crate::{ArrayVec, RunError};
use std::collections::HashMap;

use std::time::UNIX_EPOCH;

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
        let mut ret = Self {
            frames: ArrayVec::new(),
            stack: ArrayVec::new(),
            globals: HashMap::new(),
            objects: Manager::new(),
        };
        ret.define_native("clock", 0, |_| { Value::Number(UNIX_EPOCH.elapsed().unwrap().as_millis() as f64) });
        ret
    }

    pub fn define_native(&mut self, name: &str, arity: u8, f: fn(&[Value]) -> Value) {
        let name = self.objects.new_string(name.to_owned());
        self.stack.push(Value::String(name));
        self.globals.insert(
            name,
            Value::NativeFunction {
                arity,
                f,
            }
        );
        self.stack.pop();
    }

    pub fn run(&mut self, source: &str) -> Result<(), ()> {
        let function = Compiler::with_lexer(&mut Lexer::new(source).peekable(), &mut self.objects, FunctionKind::Script).compile()?;
        let function = self.objects.new_function(function);

        self.frames.push(CallFrame::new(function, 0));
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
        let function = self.current_frame().function;
        let function = self.objects.get_function(function);
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
            macro_rules! function {
                () => {{
                    let function = self.frames.last().unwrap().function;
                    self.objects.get_function(function)
                }};
            }
            macro_rules! read_byte {
                () => {{
                    let frame = self.frames.last_mut().unwrap();
                    let function = self.objects.get_function(frame.function);
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
                () => {
                    function!().chunk.get_constant(read_byte!())
                };
            }
            // TODO: add macro for binary expressions
            match read_byte!().try_into().unwrap() {
                OpCode::Return => {
                    let result = self.stack.pop().unwrap();
                    let frame = self.frames.pop().unwrap();
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
                    let constant = read_constant!().clone();
                    self.stack.push(constant);
                }
                OpCode::Add => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.stack.push(Value::Number(a + b))
                        }
                        (Value::String(a), Value::String(b)) => {
                            let value =
                                self.objects.get_string(a).to_owned() + self.objects.get_string(b);
                            let value = self.objects.new_string(value);
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
                    let value = self.stack.last().unwrap().clone();
                    self.print_value(value);
                    println!("");
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
                    let value = self.stack[slot as usize].clone();
                    self.stack.push(value);
                }
                OpCode::SetLocal => {
                    let slot = read_byte!();
                    let slot = self.current_frame().base + slot as usize;
                    let value = self.stack.last().unwrap().clone();
                    self.stack[slot as usize] = value;
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
                OpCode::Call => {
                    let args_count = read_byte!();
                    let base = self.stack.len() - 1 - args_count as usize;
                    let function = self.stack[base].clone();
                    match function {
                        Value::Function(f) => {
                            let arity = self.objects.get_function(f).arity;
                            if arity != args_count {
                                self.error(RunError::WrongArity(arity, args_count))?;
                            }
                            self.frames.push(CallFrame::new(f, base));
                        }
                        Value::NativeFunction{ arity, f } => {
                            if arity != args_count {
                                self.error(RunError::WrongArity(arity, args_count))?;
                            }
                            let result = f(&self.stack[base + 1..]);
                            for _ in 0..args_count + 1 {
                                self.stack.pop();
                            }
                            self.stack.push(result);
                        }
                        _ => self.error(RunError::NotCallable)?,
                    }
                }
            }
        }
    }

    fn print_value(&mut self, value: Value) {
        self.objects.print_value(value);
    }
}
