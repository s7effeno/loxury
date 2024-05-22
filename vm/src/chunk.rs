use gc::{Finalize, Gc, Trace};
use std::fmt::{self, Display};

use crate::location::Coords;

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

    pub fn byte_at(&self, index: usize) -> u8 {
        self.code[index]
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

    pub fn get_constant(&self, index: u8) -> Value {
        self.constants[index as usize].clone()
    }

    pub fn coords(&self, index: usize) -> Coords {
        self.coords[index]
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut bytes = self.code.iter();
        while let Some(&b) = bytes.next() {
            match b.try_into().unwrap() {
                OpCode::Constant => {
                    let index = *bytes.next().unwrap() as usize;
                    let value = &self.constants[index];
                    writeln!(f, "constant {value}")?;
                }
                OpCode::Add => {
                    writeln!(f, "add")?;
                }
                OpCode::Subtract => {
                    writeln!(f, "subtract")?;
                }
                OpCode::Multiply => {
                    writeln!(f, "multiply")?;
                }
                OpCode::Divide => {
                    writeln!(f, "divide")?;
                }
                OpCode::Negate => {
                    writeln!(f, "negate")?;
                }
                OpCode::Return => {
                    writeln!(f, "return")?;
                }
                OpCode::Nil => {
                    writeln!(f, "nil")?;
                }
                OpCode::True => {
                    writeln!(f, "true")?;
                }
                OpCode::False => {
                    writeln!(f, "false")?;
                }
                OpCode::Not => {
                    writeln!(f, "not")?;
                }
                OpCode::Equal => {
                    writeln!(f, "equal")?;
                }
                OpCode::Greater => {
                    writeln!(f, "greater")?;
                }
                OpCode::Less => {
                    writeln!(f, "less")?;
                }
            }
        }
        write!(f, "")
    }
}

pub enum OpCode {
    Constant,
    Nil,
    True,
    False,
    Equal,
    Greater,
    Less,
    Add,
    Subtract,
    Multiply,
    Divide,
    Not,
    Negate,
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
            4 => Ok(Self::Equal),
            5 => Ok(Self::Greater),
            6 => Ok(Self::Less),
            7 => Ok(Self::Add),
            8 => Ok(Self::Subtract),
            9 => Ok(Self::Multiply),
            10 => Ok(Self::Divide),
            11 => Ok(Self::Not),
            12 => Ok(Self::Negate),
            13 => Ok(Self::Return),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
    Object(Gc<Object>),
}

impl From<Object> for Value {
    fn from(value: Object) -> Self {
        Self::Object(Gc::new(value))
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(v) => write!(f, "{v}"),
            Value::Number(v) => write!(f, "{v}"),
            Value::Object(v) => write!(f, "{v}"),
        }
    }
}

#[derive(Clone, Debug, Trace, Finalize)]
pub enum Object {
    String(String),
}

impl Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(s) => write!(f, "{}", s),
        }
    }
}
