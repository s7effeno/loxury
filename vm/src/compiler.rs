use crate::chunk::{Chunk, OpCode, Value};
use crate::lex::{Lexer, Token, TokenKind};
use crate::location::{AtCoords, AtCoordsOrEof, Coords};
use crate::CompileError;

use std::iter::Peekable;

pub struct Compiler<'a> {
    lexer: Peekable<Lexer<'a>>,
    compiling_chunk: &'a mut Chunk,
    had_error: bool,
    panic_mode: bool,
}

impl<'a> Compiler<'a> {
    pub fn compile(source: &'a str, chunk: &'a mut Chunk) -> Result<(), ()> {
        let mut compiler = Self {
            lexer: Lexer::new(source).peekable(),
            compiling_chunk: chunk,
            had_error: false,
            panic_mode: false,
        };
        compiler.expression();
        if !compiler.had_error {
            compiler.current_chunk().write_nowhere(OpCode::Return as u8);
            println!("compiled: {:?}", compiler.compiling_chunk.code);
            Ok(())
        } else {
            Err(())
        }
    }

    fn next_token<'b>(&'b mut self) -> Option<AtCoords<Token<'a>>> {
        let next = self.lexer.next()?;
        match next {
            Ok(t) => Some(t),
            Err(e) => {
                self.error(&e);
                self.next_token()
            }
        }
    }

    fn peek_token<'b>(&'b mut self) -> Option<AtCoords<Token<'a>>> {
        let peek = self.lexer.peek()?;
        match peek {
            Ok(t) => Some(t.clone()),
            Err(e) => {
                let e = e.clone();
                // FIXME: can't figure out lifetimes
                self.error(&e);
                self.lexer.next();
                self.peek_token()
            }
        }
    }

    fn next_token_if<'b, F>(&'b mut self, f: F) -> Option<AtCoords<Token<'a>>>
    where
        F: Fn(TokenKind) -> bool,
    {
        self.peek_token().filter(|t| f(t.kind())).map(|t| {
            self.lexer.next();
            t
        })
    }

    fn consume(&mut self, kind: TokenKind, error: CompileError) -> Option<AtCoords<Token<'_>>> {
        let peek = self.peek_token()?;
        if kind == peek.kind() {
            let ret = peek.clone();
            self.lexer.next();
            Some(ret)
        } else {
            self.error(&peek.co_locate(error));
            None
        }
    }

    fn current_chunk(&mut self) -> &mut Chunk {
        &mut self.compiling_chunk
    }

    pub fn emit_op(&mut self, op: OpCode, coords: Coords) {
        self.emit_byte(op as u8, coords)
    }

    fn emit_bytes(&mut self, byte1: u8, byte2: u8, coords: Coords) {
        self.emit_byte(byte1, coords);
        self.emit_byte(byte2, coords);
    }

    fn emit_byte(&mut self, byte: u8, coords: Coords) {
        self.current_chunk().write(byte, coords)
    }

    fn emit_constant(&mut self, value: Value, coords: Coords) {
        let constant = self.make_constant(value);
        self.emit_bytes(OpCode::Constant as u8, constant, coords);
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

    fn error(&mut self, error: &AtCoordsOrEof<CompileError>) {
        if !self.panic_mode {
            self.panic_mode = true;
            self.had_error = true;
            eprintln!("{error}");
        }
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn parse_precedence<'b>(&'b mut self, precedence: Precedence) {
        if let Some(token) = self.next_token() {
            if self.prefix_rule(&token).is_none() {
                self.error(&token.co_locate(CompileError::ExpectedExpression));
            } else {
                while let Some(token) = self.next_token_if(|t| precedence < Self::precedence(t)) {
                    println!("accepted {:?}", token.kind());
                    self.infix_rule(&token).unwrap();
                }
            }
        } else {
            self.error(&AtCoordsOrEof::Eof(CompileError::ExpectedExpression));
        }
    }

    fn prefix_rule<'b>(&'b mut self, token: &AtCoords<Token<'a>>) -> Option<()> {
        match token.kind() {
            TokenKind::LeftParen => self.grouping(token),
            TokenKind::Minus => self.unary(token),
            TokenKind::Number => self.number(token),
            _ => return None,
        };
        Some(())
    }

    fn infix_rule<'b>(&'b mut self, token: &AtCoords<Token<'a>>) -> Option<()> {
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

    fn number(&mut self, token: &AtCoords<Token<'_>>) {
        self.emit_constant(Value::Number(token.span().parse().unwrap()), token.coords());
    }

    fn grouping<'b>(&'b mut self, _token: &AtCoords<Token<'a>>) {
        self.expression();
        self.consume(TokenKind::RightParen, CompileError::UnclosedGrouping);
    }

    fn unary<'b>(&'b mut self, token: &AtCoords<Token<'a>>) {
        self.parse_precedence(Precedence::Unary);
        match token.kind() {
            TokenKind::Minus => self.emit_op(OpCode::Subtract, token.coords()),
            _ => unreachable!(),
        }
    }

    fn binary<'b>(&'b mut self, token: &AtCoords<Token<'a>>) {
        let operator = token.kind();
        println!("parsing binary with {:?} token, {:?} precedence", token.kind(), Self::precedence(operator).next());
        self.parse_precedence(Self::precedence(operator).next());
        let op = match operator {
            TokenKind::Plus => OpCode::Add,
            TokenKind::Minus => OpCode::Subtract,
            TokenKind::Star => OpCode::Multiply,
            TokenKind::Slash => OpCode::Divide,
            _ => unreachable!(),
        };
        self.emit_op(op, token.coords())
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
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
