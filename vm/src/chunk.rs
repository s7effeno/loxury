use std::fmt::{self, Display};

use crate::gc::{Gc, GcHandle, Manager};
use crate::location::Coords;

#[derive(Debug)]
pub struct Chunk {
    code: Vec<u8>,
    coords: Vec<Coords>,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            coords: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.code.len()
    }

    // TODO: consider making unsafe, +3% boost
    pub fn byte_at(&self, index: usize) -> u8 {
        self.code[index]
    }

    pub fn at_mut(&mut self, index: usize) -> &mut u8 {
        &mut self.code[index]
    }

    pub fn write(&mut self, byte: u8, coords: Coords) {
        self.code.push(byte);
        self.coords.push(coords);
    }

    pub fn write_nowhere(&mut self, byte: u8) {
        self.code.push(byte)
    }

    pub fn add_constant(&mut self, value: Value) -> u32 {
        self.constants.push(value);
        (self.constants.len() - 1) as u32
    }

    pub fn get_constant(&self, index: u8) -> &Value {
        &self.constants[index as usize]
    }

    pub fn coords(&self, index: usize) -> Coords {
        self.coords[index]
    }

    pub fn disassemble(&self, objects: &Manager) -> fmt::Result {
        let mut bytes = self.code.iter().enumerate();
        macro_rules! simple {
            ($op_name:expr) => {
                println!("{}", $op_name)
            };
        }
        macro_rules! byte {
            ($op_name:expr) => {{
                let arg = *bytes.next().unwrap().1;
                println!("{} {}", $op_name, arg)
            }};
        }
        macro_rules! constant {
            ($op_name:expr) => {{
                let index = *bytes.next().unwrap().1 as usize;
                let arg = &self.constants[index];
                print!("{} {} ", $op_name, index);
                objects.print_value(arg.clone());
                println!("");
            }};
        }
        macro_rules! jump {
            ($op_name:expr, $addr:expr, $sign:tt) => {{
                let offset = u16::from_be_bytes([*bytes.next().unwrap().1, *bytes.next().unwrap().1]);
                let destination = $addr + 3 $sign offset as usize;
                println!("{} {} -> {}", $op_name, offset, destination)
            }};
        }
        while let Some((addr, &b)) = bytes.next() {
            print!("{} ", addr);
            match b.try_into().unwrap() {
                OpCode::Constant => {
                    constant!("constant");
                }
                OpCode::Add => {
                    simple!("add");
                }
                OpCode::Subtract => {
                    simple!("subtract");
                }
                OpCode::Multiply => {
                    simple!("multiply");
                }
                OpCode::Divide => {
                    simple!("divide");
                }
                OpCode::Negate => {
                    simple!("negate");
                }
                OpCode::Return => {
                    simple!("return");
                }
                OpCode::Nil => {
                    simple!("nil");
                }
                OpCode::True => {
                    simple!("true");
                }
                OpCode::False => {
                    simple!("false");
                }
                OpCode::Not => {
                    simple!("not");
                }
                OpCode::Equal => {
                    simple!("equal");
                }
                OpCode::Greater => {
                    simple!("greater");
                }
                OpCode::Less => {
                    simple!("less");
                }
                OpCode::Print => {
                    simple!("print");
                }
                OpCode::Pop => {
                    simple!("pop");
                }
                OpCode::DefineGlobal => {
                    constant!("define global");
                }
                OpCode::GetGlobal => {
                    constant!("get global");
                }
                OpCode::SetGlobal => {
                    constant!("set global");
                }
                OpCode::GetLocal => {
                    byte!("get local");
                }
                OpCode::SetLocal => {
                    byte!("set local");
                }
                OpCode::JumpIfFalse => {
                    jump!("jump if false", addr, +);
                }
                OpCode::Jump => {
                    jump!("jump", addr, +);
                }
                OpCode::Loop => {
                    jump!("loop", addr, -);
                }
                OpCode::Call => {
                    byte!("call")
                }
                OpCode::Closure => {
                    constant!("closure")
                }
                OpCode::GetUpvalue => {
                    byte!("get upvalue")
                }
                OpCode::SetUpvalue => {
                    byte!("set upvalue")
                }
            }
        }
        Ok(())
    }
}

pub enum OpCode {
    Constant,
    Nil,
    True,
    False,
    Pop,
    GetLocal,
    SetLocal,
    GetGlobal,
    DefineGlobal,
    SetGlobal,
    GetUpvalue,
    SetUpvalue,
    Equal,
    Greater,
    Less,
    Add,
    Subtract,
    Multiply,
    Divide,
    Not,
    Negate,
    Print,
    Jump,
    JumpIfFalse,
    Loop,
    Call,
    Closure,
    Return,
}

impl TryFrom<u8> for OpCode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Constant),
            1 => Ok(Self::Nil),
            2 => Ok(Self::True),
            3 => Ok(Self::False),
            4 => Ok(Self::Pop),
            5 => Ok(Self::GetLocal),
            6 => Ok(Self::SetLocal),
            7 => Ok(Self::GetGlobal),
            8 => Ok(Self::DefineGlobal),
            9 => Ok(Self::SetGlobal),
            10 => Ok(Self::GetUpvalue),
            11 => Ok(Self::SetUpvalue),
            12 => Ok(Self::Equal),
            13 => Ok(Self::Greater),
            14 => Ok(Self::Less),
            15 => Ok(Self::Add),
            16 => Ok(Self::Subtract),
            17 => Ok(Self::Multiply),
            18 => Ok(Self::Divide),
            19 => Ok(Self::Not),
            20 => Ok(Self::Negate),
            21 => Ok(Self::Print),
            22 => Ok(Self::Jump),
            23 => Ok(Self::JumpIfFalse),
            24 => Ok(Self::Loop),
            25 => Ok(Self::Call),
            26 => Ok(Self::Closure),
            27 => Ok(Self::Return),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
    String(GcHandle<String>),
    Function(GcHandle<Function>),
    // TODO: ensure not wrapping is ok
    NativeFunction { arity: u8, f: fn(&[Value]) -> Value },
    Closure(GcHandle<Closure>),
}

impl Value {
    pub fn try_as_string(&self) -> Result<GcHandle<String>, ()> {
        if let Self::String(v) = self {
            Ok(*v)
        } else {
            Err(())
        }
    }

    pub fn try_as_function(&self) -> Result<GcHandle<Function>, ()> {
        if let Self::Function(v) = self {
            Ok(*v)
        } else {
            Err(())
        }
    }
}

#[derive(Debug)]
pub struct Function {
    pub arity: u8,
    pub chunk: Chunk,
    pub name: Option<String>,
    pub kind: FunctionKind,
    // FIXME: u8?
    pub upvalue_count: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum FunctionKind {
    Function,
    Script,
}

impl Function {
    pub fn new(kind: FunctionKind) -> Self {
        Self {
            arity: 0,
            chunk: Chunk::new(),
            name: None,
            upvalue_count: 0,
            kind,
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "<fn {}>", name),
            None => write!(f, "<script>"),
        }
    }
}

#[derive(Debug)]
pub struct Closure {
    pub function: GcHandle<Function>,
    pub upvalues: Vec<ObjUpvalue>,
}

impl Closure {
    pub fn new(function: GcHandle<Function>) -> Self {
        Self { function, upvalues: Vec::new()}
    }
}

#[derive(Ord, PartialOrd, PartialEq, Eq)]
pub struct Upvalue {
    index: u8,
    is_local: bool,
}

impl Upvalue {
    pub fn new(index: u8, is_local: bool) -> Self {
        Self { index, is_local }
    }
}

#[derive(Debug)]
pub struct ObjUpvalue {
    value: Value,
}
