use crate::chunk::{Chunk, OpCode, Value};
use crate::lex::{Lexer, Token, TokenKind};
use crate::{CompileError, Located};

use std::iter::Peekable;

struct Compiler<'a> {
    lexer: Peekable<Lexer<'a>>,
    chunks: Vec<Chunk>,
    chunk: usize,
}

impl<'a> Compiler<'a> {
    fn next_token(&mut self) -> Option<Located<Token<'_>>> {
        let next = self.lexer.next()?;
        match next {
            Ok(t) => Some(t),
            Err(e) => {
                eprintln!("{e}");
                self.next_token()
            }
        }
    }

    fn peek_token<'b>(&'b mut self) -> Option<Located<Token<'a>>> {
        let peek = self.lexer.peek()?;
        match peek {
            Ok(t) => Some(t.clone()),
            Err(e) => {
                eprintln!("{e}");
                self.lexer.next();
                self.peek_token()
            }
        }
    }

    fn consume(&mut self, kind: TokenKind, error: CompileError) -> Option<Located<Token<'_>>> {
        let peek = self.peek_token()?;
        if kind == peek.kind() {
            let ret = peek.clone();
            self.lexer.next();
            Some(ret)
        } else {
            None
        }
    }

    fn current_chunk(&mut self) -> &mut Chunk {
        &mut self.chunks[self.chunk]
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

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

    fn number(&mut self, token: Located<Token<'_>>) {
        self.emit_constant(
            Value::Number(token.span().parse().unwrap()),
            (token.row(), token.col()),
        );
    }

    fn grouping(&'a mut self, token: Located<Token<'_>>) {
        self.expression();
        self.consume(TokenKind::RightParen, CompileError::UnclosedGrouping);
    }

    fn unary(&mut self, token: Located<Token<'_>>) {
        self.parse_precedence(Precedence::Unary);
        match token.kind() {
            TokenKind::Minus => self.emit_op(OpCode::Subtract, (token.row(), token.col())),
            _ => unreachable!(),
        }
    }

    fn binary(&mut self, token: Located<Token<'_>>) {
        let operator = token.kind();
        self.parse_precedence(Self::precedence(operator));
        match operator {
            TokenKind::Plus => self.emit_op(OpCode::Add, (token.row(), token.col())),
            TokenKind::Minus => self.emit_op(OpCode::Subtract, (token.row(), token.col())),
            TokenKind::Star => self.emit_op(OpCode::Multiply, (token.row(), token.col())),
            TokenKind::Slash => self.emit_op(OpCode::Divide, (token.row(), token.col())),
            _ => unreachable!(),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        todo!()
    }

    fn prefix_rule(&'a mut self, token: Located<Token<'_>>) -> Option<()> {
        match token.kind() {
            TokenKind::LeftParen => self.grouping(token),
            TokenKind::Minus => self.unary(token),
            TokenKind::Number => self.number(token),
            _ => return None,
        };
        Some(())
    }

    fn infix_rule(&'a mut self, token: Located<Token<'_>>) -> Option<()> {
        match token.kind() {
            TokenKind::Minus => self.binary(token),
            TokenKind::Plus => self.binary(token),
            TokenKind::Slash => self.binary(token),
            TokenKind::Star => self.binary(token),
            _ => return None,
        };
        Some(())
    }

    fn precedence(kind: TokenKind) -> Precedence {
        match kind {
            TokenKind::LeftParen => Precedence::None,
            TokenKind::RightParen => Precedence::None,
            TokenKind::LeftBrace => Precedence::None,
            TokenKind::RightBrace => Precedence::None,
            TokenKind::Comma => Precedence::None,
            TokenKind::Dot => Precedence::None,
            TokenKind::Minus => Precedence::Term,
            TokenKind::Plus => Precedence::Term,
            TokenKind::Semicolon => Precedence::None,
            TokenKind::Slash => Precedence::Factor,
            TokenKind::Star => Precedence::Factor,
            TokenKind::Bang => Precedence::None,
            TokenKind::BangEqual => Precedence::None,
            TokenKind::Equal => Precedence::None,
            TokenKind::EqualEqual => Precedence::None,
            TokenKind::Greater => Precedence::None,
            TokenKind::GreaterEqual => Precedence::None,
            TokenKind::Less => Precedence::None,
            TokenKind::LessEqual => Precedence::None,
            TokenKind::And => Precedence::None,
            TokenKind::Class => Precedence::None,
            TokenKind::Else => Precedence::None,
            TokenKind::False => Precedence::None,
            TokenKind::Fun => Precedence::None,
            TokenKind::For => Precedence::None,
            TokenKind::If => Precedence::None,
            TokenKind::Nil => Precedence::None,
            TokenKind::Or => Precedence::None,
            TokenKind::Print => Precedence::None,
            TokenKind::Return => Precedence::None,
            TokenKind::Super => Precedence::None,
            TokenKind::This => Precedence::None,
            TokenKind::True => Precedence::None,
            TokenKind::Var => Precedence::None,
            TokenKind::While => Precedence::None,
            TokenKind::String => Precedence::None,
            TokenKind::Number => Precedence::None,
            TokenKind::Identifier => Precedence::None,
        }
    }
}

enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

impl Precedence {
    fn next(&self) -> Self {
        match self {
            Precedence::None => Self::Assignment,
            Precedence::Assignment => Self::Or,
            Precedence::Or => Self::And,
            Precedence::And => Self::Equality,
            Precedence::Equality => Self::Comparison,
            Precedence::Comparison => Self::Term,
            Precedence::Term => Self::Factor,
            Precedence::Factor => Self::Unary,
            Precedence::Unary => Self::Call,
            Precedence::Call => Self::Primary,
            Precedence::Primary => Self::Primary,
        }
    }
}
