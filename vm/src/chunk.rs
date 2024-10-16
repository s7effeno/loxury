use std::fmt::{self, Display};

use crate::location::Coords;
use crate::gc::Gc;

#[derive(Debug)]
pub struct Chunk {
    code: Vec<u8>,
    coords: Vec<Coords>,
    constants: Vec<Value>,
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
}

impl Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut bytes = self.code.iter().enumerate();
        macro_rules! simple {
            ($op_name:expr) => {
                writeln!(f, "{}", $op_name)
            };
        }
        macro_rules! byte {
            ($op_name:expr) => {{
                let arg = *bytes.next().unwrap().1;
                writeln!(f, "{} {}", $op_name, arg)
            }};
        }
        macro_rules! constant {
            ($op_name:expr) => {{
                let index = *bytes.next().unwrap().1 as usize;
                let arg = &self.constants[index];
                writeln!(f, "{} {} {}", $op_name, index, arg)
            }};
        }
        macro_rules! jump {
            ($op_name:expr, $addr:expr, $sign:tt) => {{
                let offset = u16::from_be_bytes([*bytes.next().unwrap().1, *bytes.next().unwrap().1]);
                let destination = $addr + 3 $sign offset as usize;
                writeln!(f, "{} {} -> {}", $op_name, offset, destination)
            }};
        }
        while let Some((addr, &b)) = bytes.next() {
            write!(f, "{} ", addr)?;
            match b.try_into().unwrap() {
                OpCode::Constant => {
                    constant!("constant")?;
                }
                OpCode::Add => {
                    simple!("add")?;
                }
                OpCode::Subtract => {
                    simple!("subtract")?;
                }
                OpCode::Multiply => {
                    simple!("multiply")?;
                }
                OpCode::Divide => {
                    simple!("divide")?;
                }
                OpCode::Negate => {
                    simple!("negate")?;
                }
                OpCode::Return => {
                    simple!("return")?;
                }
                OpCode::Nil => {
                    simple!("nil")?;
                }
                OpCode::True => {
                    simple!("true")?;
                }
                OpCode::False => {
                    simple!("false")?;
                }
                OpCode::Not => {
                    simple!("not")?;
                }
                OpCode::Equal => {
                    simple!("equal")?;
                }
                OpCode::Greater => {
                    simple!("greater")?;
                }
                OpCode::Less => {
                    simple!("less")?;
                }
                OpCode::Print => {
                    simple!("print")?;
                }
                OpCode::Pop => {
                    simple!("pop")?;
                }
                OpCode::DefineGlobal => {
                    constant!("define global")?;
                }
                OpCode::GetGlobal => {
                    constant!("get global")?;
                }
                OpCode::SetGlobal => {
                    constant!("set global")?;
                }
                OpCode::GetLocal => {
                    byte!("get local")?;
                }
                OpCode::SetLocal => {
                    byte!("set local")?;
                }
                OpCode::JumpIfFalse => {
                    jump!("jump if false", addr, +)?;
                }
                OpCode::Jump => {
                    jump!("jump", addr, +)?;
                }
                OpCode::Loop => {
                    jump!("loop", addr, -)?;
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
            10 => Ok(Self::Equal),
            11 => Ok(Self::Greater),
            12 => Ok(Self::Less),
            13 => Ok(Self::Add),
            14 => Ok(Self::Subtract),
            15 => Ok(Self::Multiply),
            16 => Ok(Self::Divide),
            17 => Ok(Self::Not),
            18 => Ok(Self::Negate),
            19 => Ok(Self::Print),
            20 => Ok(Self::Jump),
            21 => Ok(Self::JumpIfFalse),
            22 => Ok(Self::Loop),
            23 => Ok(Self::Return),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
    String(Gc),
    Function(Gc),
}

impl Value {
    pub fn try_as_string(&self) -> Result<Gc, ()> {
        if let Self::String(v) = self {
            Ok(*v)
        } else {
            Err(())
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(v) => write!(f, "{v}"),
            Value::Number(v) => write!(f, "{v}"),
            // FIXME: this must be implemented in the vm
            Value::String(v) => write!(f, "{}", todo!()),
            Value::Function(v) => write!(f, "{}", todo!()),
        }
    }
}

#[derive(Debug)]
pub struct Function {
    pub arity: u8,
    pub chunk: Chunk,
    pub name: Gc,
}

impl Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fn {}>", todo!())
    }
}
