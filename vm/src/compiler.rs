use crate::chunk::{Chunk, OpCode, Value};
use crate::lex::{Lexer, TokenKind};
use crate::{CompileError, Located};

struct Compiler<'a> {
    lexer: Lexer<'a>,
    chunks: Vec<Chunk>,
    chunk: usize,
}

impl Compiler<'_> {
    fn consume(kind: TokenKind, error: CompileError) {
    }

    fn current_chunk(&mut self) -> &mut Chunk {
        &mut self.chunks[self.chunk]
    }

    fn expression(&mut self) {
        todo!()
    }

    fn grouping(&mut self) {
        self.expression()
    }

    fn unary(&mut self) {}

    pub fn emit_op(&mut self, op: OpCode, pos: (u16, u16)) {
        self.emit_byte(op as u8, pos)
    }

    fn emit_bytes(&mut self, byte1: u8, byte2: u8, pos: (u16, u16)) {
        self.emit_byte(byte1, pos);
        self.emit_byte(byte2, pos);
    }

    fn emit_byte(&mut self, byte: u8, pos: (u16, u16)) {
        self.current_chunk().write(byte, pos)
    }

    fn emit_constant(&mut self, value: Value, pos: (u16, u16)) {
        let constant = self.make_constant(value);
        self.emit_bytes(OpCode::Constant as u8, constant, pos);
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let constant = self.current_chunk().add_constant(value);
        if constant >= u8::MAX as u32 {
            // ERROR
            0
        } else {
            constant as u8
        }
    }

    fn error(error: Located<CompileError>) {
        eprintln!("{error}")
    }
}
