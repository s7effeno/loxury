use crate::location::Coords;

pub struct Chunk {
    code: Vec<u8>,
    coords: Vec<Coords>,
    constants: Vec<Value>,
}

impl Chunk {
    pub fn byte_at(&self, index: usize) -> u8 {
        self.code[index]
    }

    pub fn write(&mut self, byte: u8, coords: Coords) {
        self.code.push(byte);
        self.coords.push(coords);
    }

    pub fn add_constant(&mut self, value: Value) -> u32 {
        self.constants.push(value);
        (self.constants.len() - 1) as u32
    }

    pub fn get_constant(&self, index: u8) -> Value {
        self.constants[index as usize]
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
            _ => Err(())
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Value {
    Number(f64),
}
