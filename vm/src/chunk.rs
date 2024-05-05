use std::fmt::{Display, self};

use crate::location::Coords;

#[derive(Debug)]
pub struct Chunk {
    pub code: Vec<u8>,
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
        self.constants[index as usize]
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
                OpCode::Constant => todo!(),
                OpCode::Add => todo!(),
                OpCode::Subtract => todo!(),
                OpCode::Multiply => todo!(),
                OpCode::Divide => todo!(),
                OpCode::Negate => todo!(),
                OpCode::Return => todo!(),
            }
        }
        todo!()
    }
}

pub enum OpCode {
    Constant,
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Return,
}

impl TryFrom<u8> for OpCode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Constant),
            1 => Ok(Self::Add),
            2 => Ok(Self::Subtract),
            3 => Ok(Self::Multiply),
            4 => Ok(Self::Divide),
            5 => Ok(Self::Negate),
            6 => Ok(Self::Return),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
}
