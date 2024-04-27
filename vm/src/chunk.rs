pub struct Chunk {
    code: Vec<u8>,
    lines: Vec<(u16, u16)>,
    constants: Vec<Value>,
}

impl Chunk {
    pub fn write(&mut self, byte: u8, pos: (u16, u16)) {
        self.code.push(byte);
        self.lines.push(pos);
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
