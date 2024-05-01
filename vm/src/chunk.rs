use crate::location::Coords;

pub struct Chunk {
    code: Vec<u8>,
    coords: Vec<Coords>,
    constants: Vec<Value>,
}

impl Chunk {
    pub fn write(&mut self, byte: u8, coords: Coords) {
        self.code.push(byte);
        self.coords.push(coords);
    }

    pub fn add_constant(&mut self, value: Value) -> u32 {
        self.constants.push(value);
        (self.constants.len() - 1) as u32
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

pub enum Value {
    Number(f64),
}
