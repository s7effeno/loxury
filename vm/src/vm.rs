use crate::chunk::{Chunk, Value};

struct Vm {
    chunks: Chunk,
    ip: usize,
    // stack: [Value;
}

impl Vm {
    fn read_byte(&mut self) -> u8 {
        // let ret = self.chunks[self.ip];
        self.ip += 1;
        // ret
        todo!()
    }
}
