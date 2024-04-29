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
        // let rule = self.get_rule(operator);
        // self.parse_precedence(rule as u8 + 1);
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        todo!()
    }

    fn get_rule(kind: TokenKind) -> ParseRule {
        fn unreachable(_: &mut Compiler, _: Located<Token<'_>>) {
            unreachable!()
        }

        match kind {
            TokenKind::LeftParen => ParseRule(Self::grouping, Self::grouping, Precedence::Or),
            TokenKind::RightParen => todo!(),
            TokenKind::LeftBrace => todo!(),
            TokenKind::RightBrace => todo!(),
            TokenKind::Comma => todo!(),
            TokenKind::Dot => todo!(),
            TokenKind::Minus => todo!(),
            TokenKind::Plus => todo!(),
            TokenKind::Semicolon => todo!(),
            TokenKind::Slash => todo!(),
            TokenKind::Star => todo!(),
            TokenKind::Bang => todo!(),
            TokenKind::BangEqual => todo!(),
            TokenKind::Equal => todo!(),
            TokenKind::EqualEqual => todo!(),
            TokenKind::Greater => todo!(),
            TokenKind::GreaterEqual => todo!(),
            TokenKind::Less => todo!(),
            TokenKind::LessEqual => todo!(),
            TokenKind::And => todo!(),
            TokenKind::Class => todo!(),
            TokenKind::Else => todo!(),
            TokenKind::False => todo!(),
            TokenKind::Fun => todo!(),
            TokenKind::For => todo!(),
            TokenKind::If => todo!(),
            TokenKind::Nil => todo!(),
            TokenKind::Or => todo!(),
            TokenKind::Print => todo!(),
            TokenKind::Return => todo!(),
            TokenKind::Super => todo!(),
            TokenKind::This => todo!(),
            TokenKind::True => todo!(),
            TokenKind::Var => todo!(),
            TokenKind::While => todo!(),
            TokenKind::String => todo!(),
            TokenKind::Number => todo!(),
            TokenKind::Identifier => todo!(),
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

type ParseFn = for<'a, 'b> fn(&'a mut Compiler<'_>, Located<Token<'b>>);

struct ParseRule(ParseFn, ParseFn, Precedence);
