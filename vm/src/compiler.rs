use crate::chunk::{Chunk, Object, OpCode, Value};
use crate::lex::{Lexer, Token, TokenKind};
use crate::location::{AtCoords, AtCoordsOrEof, Coords};
use crate::CompileError;

use std::iter::Peekable;

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

pub struct Compiler<'a> {
    lexer: Peekable<Lexer<'a>>,
    compiling_chunk: &'a mut Chunk,
    had_error: bool,
    panic_mode: bool,
}

impl<'a> Compiler<'a> {
    fn error(&mut self, error: &AtCoordsOrEof<CompileError>) {
        if !self.panic_mode {
            self.panic_mode = true;
            self.had_error = true;
            eprintln!("{error}");
        }
    }

    fn synchronize(&mut self) {
        while let Some(t) = self.peek_token() {
            match t.kind() {
                TokenKind::Class
                | TokenKind::Fun
                | TokenKind::Var
                | TokenKind::For
                | TokenKind::If
                | TokenKind::While
                | TokenKind::Print
                | TokenKind::Return => break,
                TokenKind::Semicolon => {
                    self.lexer.next();
                    break;
                }
                _ => {
                    self.next_token();
                }
            }
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

    fn next_token_if_eq<'b>(&'b mut self, kind: TokenKind) -> Option<AtCoords<Token<'_>>> {
        self.next_token_if(|t| t == kind)
    }

    fn consume(&mut self, kind: TokenKind, error: CompileError) -> Option<AtCoords<Token<'_>>> {
        let peek = self.peek_token();
        if let Some(peek) = peek {
            if peek.kind() == kind {
                let ret = peek.clone();
                self.lexer.next();
                Some(ret)
            } else {
                self.error(&peek.co_locate(error));
                None
            }
        } else {
            self.error(&AtCoordsOrEof::Eof(error));
            None
        }
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

    pub fn compile(source: &'a str, chunk: &'a mut Chunk) -> Result<(), ()> {
        let mut compiler = Self {
            lexer: Lexer::new(source).peekable(),
            compiling_chunk: chunk,
            had_error: false,
            panic_mode: false,
        };
        while compiler.peek_token().is_some() {
            compiler.declaration();
        }
        if !compiler.had_error {
            compiler.current_chunk().write_nowhere(OpCode::Return as u8);
            Ok(())
        } else {
            Err(())
        }
    }

    fn current_chunk(&mut self) -> &mut Chunk {
        &mut self.compiling_chunk
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

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn var_declaration(&mut self) {
        if let Ok((index, coords)) = self.parse_variable(CompileError::ExpectedVariableName) {
            if self.next_token_if_eq(TokenKind::Equal).is_some() {
                self.expression();
            } else {
                self.emit_op(OpCode::Nil, coords);
            }
            if let Some(coords) = self
                .consume(TokenKind::Semicolon, CompileError::UnclosedStatement)
                .map(|t| t.coords())
            {
                self.define_variable(index, coords);
            }
        }
    }

    fn declaration(&mut self) {
        if self.next_token_if_eq(TokenKind::Var).is_some() {
            self.var_declaration()
        } else {
            self.statement();
        }

        if self.panic_mode {
            self.synchronize();
        }
    }

    fn statement(&mut self) {
        if self.next_token_if_eq(TokenKind::Print).is_some() {
            self.print_statement();
        } else {
            self.expression_statement();
        }
    }

    fn print_statement(&mut self) {
        self.expression();
        if let Some(c) = self
            .consume(TokenKind::Semicolon, CompileError::UnclosedStatement)
            .map(|t| t.coords())
        {
            self.emit_op(OpCode::Print, c)
        }
    }

    fn identifier_constant(&mut self, name: String) -> u8 {
        self.make_constant(Value::Object(Object::String(name.into())))
    }

    fn parse_variable(&mut self, error: CompileError) -> Result<(u8, Coords), ()> {
        if let Some(identifier) = self.consume(TokenKind::Identifier, error) {
            let span = identifier.span().into();
            let coords = identifier.coords();
            Ok((self.identifier_constant(span), coords))
        } else {
            Err(())
        }
    }

    fn expression_statement(&mut self) {
        self.expression();
        if let Some(c) = self
            .consume(TokenKind::Semicolon, CompileError::UnclosedStatement)
            .map(|t| t.coords())
        {
            self.emit_op(OpCode::Pop, c);
        }
    }

    fn define_variable(&mut self, global: u8, coords: Coords) {
        self.emit_op(OpCode::DefineGlobal, coords);
        self.emit_byte(global, coords);
    }

    fn parse_precedence<'b>(&'b mut self, precedence: Precedence) {
        if let Some(token) = self.next_token() {
            let can_assign = precedence <= Precedence::Assignment;
            if self.prefix_rule(&token, can_assign).is_some() {
                while let Some(token) = self.next_token_if(|t| precedence <= Self::precedence(t)) {
                    self.infix_rule(&token).unwrap();
                }

                if let Some(coords) = self
                    .next_token_if(|t| can_assign && t == TokenKind::Equal)
                    .map(|t| t.coords())
                {
                    self.error(&coords.locate(CompileError::InvalidAssignmentTarget).into())
                }
            } else {
                self.error(&token.co_locate(CompileError::ExpectedExpression));
            }
        } else {
            self.error(&AtCoordsOrEof::Eof(CompileError::ExpectedExpression));
        }
    }

    fn prefix_rule<'b>(&'b mut self, token: &AtCoords<Token<'a>>, can_assign: bool) -> Option<()> {
        match token.kind() {
            TokenKind::LeftParen => self.grouping(token),
            TokenKind::Minus => self.unary(token),
            TokenKind::Number => self.number(token),
            TokenKind::False => self.literal(token),
            TokenKind::True => self.literal(token),
            TokenKind::Nil => self.literal(token),
            TokenKind::Bang => self.unary(token),
            TokenKind::String => self.string(token),
            TokenKind::Identifier => self.variable(token, can_assign),
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
            TokenKind::BangEqual => self.binary(token),
            TokenKind::EqualEqual => self.binary(token),
            TokenKind::Greater => self.binary(token),
            TokenKind::GreaterEqual => self.binary(token),
            TokenKind::Less => self.binary(token),
            TokenKind::LessEqual => self.binary(token),
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
            TokenKind::BangEqual => Precedence::Equality,
            TokenKind::Equal => Precedence::None,
            TokenKind::EqualEqual => Precedence::Equality,
            TokenKind::Greater => Precedence::Comparison,
            TokenKind::GreaterEqual => Precedence::Comparison,
            TokenKind::Less => Precedence::Comparison,
            TokenKind::LessEqual => Precedence::Comparison,
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

    fn string(&mut self, token: &AtCoords<Token<'_>>) {
        self.emit_constant(
            Object::String(token.span().to_owned().into()).into(),
            token.coords(),
        )
    }

    fn named_variable(&mut self, token: &AtCoords<Token<'_>>, can_assign: bool) {
        let arg = self.identifier_constant(token.span().into());
        match self
            .next_token_if(|t| can_assign && t == TokenKind::Equal)
            .map(|t| t.coords())
        {
            Some(coords) => {
                self.expression();
                self.emit_op(OpCode::SetGlobal, coords);
                self.emit_byte(arg, coords);
            }
            _ => {
                self.emit_op(OpCode::GetGlobal, token.coords());
                self.emit_byte(arg, token.coords());
            }
        }
    }

    fn variable(&mut self, token: &AtCoords<Token<'_>>, can_assign: bool) {
        self.named_variable(token, can_assign)
    }

    fn grouping<'b>(&'b mut self, _token: &AtCoords<Token<'a>>) {
        self.expression();
        self.consume(TokenKind::RightParen, CompileError::UnclosedGrouping);
    }

    fn unary<'b>(&'b mut self, token: &AtCoords<Token<'a>>) {
        self.parse_precedence(Precedence::Unary);
        match token.kind() {
            TokenKind::Bang => self.emit_op(OpCode::Not, token.coords()),
            TokenKind::Minus => self.emit_op(OpCode::Subtract, token.coords()),
            _ => unreachable!(),
        }
    }

    fn binary<'b>(&'b mut self, token: &AtCoords<Token<'a>>) {
        let operator = token.kind();
        self.parse_precedence(Self::precedence(operator).next());
        match operator {
            TokenKind::Plus => self.emit_op(OpCode::Add, token.coords()),
            TokenKind::Minus => self.emit_op(OpCode::Subtract, token.coords()),
            TokenKind::Star => self.emit_op(OpCode::Multiply, token.coords()),
            TokenKind::Slash => self.emit_op(OpCode::Divide, token.coords()),
            TokenKind::BangEqual => {
                self.emit_op(OpCode::Equal, token.coords());
                self.emit_op(OpCode::Not, token.coords());
            }
            TokenKind::EqualEqual => self.emit_op(OpCode::Equal, token.coords()),
            TokenKind::Greater => self.emit_op(OpCode::Greater, token.coords()),
            TokenKind::GreaterEqual => {
                self.emit_op(OpCode::Less, token.coords());
                self.emit_op(OpCode::Not, token.coords());
            }
            TokenKind::Less => self.emit_op(OpCode::Less, token.coords()),
            TokenKind::LessEqual => {
                self.emit_op(OpCode::Greater, token.coords());
                self.emit_op(OpCode::Not, token.coords());
            }
            _ => unreachable!(),
        }
    }

    fn literal<'b>(&'b mut self, token: &AtCoords<Token<'a>>) {
        match token.kind() {
            TokenKind::False => self.emit_op(OpCode::False, token.coords()),
            TokenKind::Nil => self.emit_op(OpCode::Nil, token.coords()),
            TokenKind::True => self.emit_op(OpCode::True, token.coords()),
            _ => unreachable!(),
        }
    }
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
