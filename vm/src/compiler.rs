// TODO: automate `emit_...` to avoid passing `coords`

use crate::chunk::{Chunk, OpCode, Value};
use crate::gc::{Gc, Manager};
use crate::lex::{Lexer, Token, TokenKind};
use crate::location::{AtCoords, AtCoordsOrEof, Coords};
use crate::CompileError;

use std::iter::Peekable;
use std::mem::MaybeUninit;

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

struct Locals<'a> {
    // Some if initialized, None if uninitialized, to implement self-referential initialization
    // error
    locals: [MaybeUninit<(&'a str, Option<usize>)>; u8::MAX as usize + 1],
    count: usize,
    scope_depth: usize,
}

impl<'a> Locals<'a> {
    fn iter<'b>(&'b self) -> impl Iterator<Item = (&'a str, Option<usize>)> + 'b {
        self.locals
            .iter()
            .take(self.count)
            .rev()
            .map(|l| unsafe { l.assume_init() })
    }

    fn resolve(&self, name: &str) -> Result<Option<u8>, ()> {
        self.iter()
            .enumerate()
            .find(|(_, (local, _))| local == &name)
            .map(|(p, (_, depth))| {
                if depth.is_none() {
                    None
                } else {
                    Some(self.count as u8 - 1 - p as u8)
                }
            })
            .ok_or(())
    }

    fn mark_initialized(&mut self) {
        assert!(self.count > 0);
        unsafe { self.locals[self.count - 1].assume_init_mut().1 = Some(self.scope_depth) };
    }
}

impl Locals<'_> {
    fn new() -> Self {
        Self {
            locals: unsafe { MaybeUninit::uninit().assume_init() },
            count: 0,
            scope_depth: 0,
        }
    }
}

struct Errors {
    had_error: bool,
    panic_mode: bool,
}

impl Errors {
    fn new() -> Self {
        Self {
            had_error: false,
            panic_mode: false,
        }
    }

    fn report(&mut self, error: &AtCoordsOrEof<CompileError>) {
        self.had_error = true;
        eprintln!("{error}");
    }

    fn sync(&mut self, error: &AtCoordsOrEof<CompileError>) {
        if !self.panic_mode {
            self.panic_mode = true;
            self.report(error);
        }
    }
}

pub struct Compiler<'a> {
    lexer: Peekable<Lexer<'a>>,
    objects: &'a mut Manager,
    function: Gc,
    locals: Locals<'a>,
    errors: Errors,
}

impl<'a> Compiler<'a> {
    // Ok(Gc(Function))
    pub fn compile(source: &'a str, objects: &mut Manager) -> Result<Gc, ()> {
        let function = objects.new_function();
        let mut compiler = Self {
            lexer: Lexer::new(source).peekable(),
            locals: Locals::new(),
            errors: Errors::new(),
            objects,
            function
        };
        while compiler.peek_token().is_some() {
            compiler.declaration();
        }
        if !compiler.errors.had_error {
            compiler.current_chunk().write_nowhere(OpCode::Return as u8);
            Ok(function)
        } else {
            Err(())
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
                self.errors.sync(&e);
                self.next_token()
            }
        }
    }

    fn peek_token<'b>(&'b mut self) -> Option<AtCoords<Token<'a>>> {
        let peek = self.lexer.peek()?;
        match peek {
            Ok(t) => Some(t.clone()),
            Err(e) => {
                self.errors.sync(&e);
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

    fn consume<'b>(
        &'b mut self,
        kind: TokenKind,
        error: CompileError,
    ) -> Option<AtCoords<Token<'a>>> {
        let peek = self.peek_token();
        if let Some(peek) = peek {
            if peek.kind() == kind {
                let ret = peek.clone();
                self.lexer.next();
                Some(ret)
            } else {
                self.errors.sync(&peek.co_locate(error));
                None
            }
        } else {
            self.errors.sync(&AtCoordsOrEof::Eof(error));
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

    fn emit_loop(&mut self, start: usize, coords: Coords) {
        self.emit_op(OpCode::Loop, coords);

        let offset = self.current_chunk().len() - start + 2;
        if offset > u16::MAX as usize {
            self.errors
                .sync(&coords.locate(CompileError::JumpTooWide).into());
        }

        self.emit_byte((offset & 0xff00) as u8, coords);
        self.emit_byte((offset & 0xff) as u8, coords);
    }

    fn emit_jump(&mut self, op: OpCode, coords: Coords) -> usize {
        self.emit_op(op, coords);
        self.emit_byte(0, coords);
        self.emit_byte(0, coords);
        self.current_chunk().len() - 2
    }

    fn emit_constant(&mut self, value: Value, coords: Coords) {
        let constant = self.make_constant(value);
        self.emit_bytes(OpCode::Constant as u8, constant, coords);
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.current_chunk().len() - offset - 2;

        if jump > u16::MAX as usize {
            let index = self.current_chunk().len() - 1;
            let coords = self.current_chunk().coords(index);
            self.errors
                .sync(&coords.locate(CompileError::JumpTooWide).into())
        }

        *self.current_chunk().at_mut(offset) = (jump & 0xff00) as u8;
        *self.current_chunk().at_mut(offset + 1) = (jump & 0xff) as u8;
    }

    fn current_chunk(&mut self) -> &mut Chunk {
        &mut self.objects.get_function(self.function).chunk
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

    fn begin_scope(&mut self) {
        self.locals.scope_depth += 1;
    }

    fn end_scope(&mut self, coords: Coords) {
        self.locals.scope_depth -= 1;

        let count = self
            .locals
            .iter()
            .take_while(|(_, depth)| {
                if let Some(depth) = depth {
                    *depth > self.locals.scope_depth
                } else {
                    false
                }
            })
            .count();

        for _ in 0..count {
            self.emit_op(OpCode::Pop, coords);
        }
        self.locals.count -= count;
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

        if self.errors.panic_mode {
            self.synchronize();
        }
    }

    fn statement(&mut self) {
        if self.next_token_if_eq(TokenKind::Print).is_some() {
            self.print_statement();
        } else if let Some(coords) = self.next_token_if_eq(TokenKind::If).map(|t| t.coords()) {
            self.if_statement(coords);
        } else if let Some(coords) = self.next_token_if_eq(TokenKind::While).map(|t| t.coords()) {
            self.while_statement(coords);
        } else if let Some(coords) = self.next_token_if_eq(TokenKind::For).map(|t| t.coords()) {
            self.for_statement(coords);
        } else if let Some(coords) = self
            .next_token_if_eq(TokenKind::LeftBrace)
            .map(|t| t.coords())
        {
            self.begin_scope();
            // TODO: `end_scope` should be called nonetheless
            self.block();
            self.end_scope(coords);
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

    fn while_statement(&mut self, coords: Coords) {
        let start = self.current_chunk().len();
        self.consume(TokenKind::LeftParen, CompileError::ExpectedControlLeftParen);
        self.expression();
        self.consume(
            TokenKind::RightParen,
            CompileError::ExpectedControlRightParen,
        );

        let end = self.emit_jump(OpCode::JumpIfFalse, coords);
        self.emit_op(OpCode::Pop, coords);
        self.statement();
        self.emit_loop(start, coords);

        self.patch_jump(end);
        self.emit_op(OpCode::Pop, coords);
    }

    fn identifier_constant(&mut self, name: String) -> u8 {
        self.make_constant(Value::String(self.objects.new_string(name)))
    }

    fn add_local(&mut self, name: AtCoords<Token<'a>>) {
        if self.locals.count == u8::MAX as usize + 1 {
            self.errors
                .sync(&name.co_locate(CompileError::TooManyLocals));
            return;
        }

        let local = &mut self.locals.locals[self.locals.count];
        self.locals.count += 1;
        local.write((name.span(), None));
    }

    fn declare_variable(&mut self, name: AtCoords<Token<'a>>) {
        if self.locals.scope_depth == 0 {
            return;
        }
        let span = name.span().to_owned();
        for (local_name, depth) in self.locals.iter() {
            match depth {
                Some(depth) if depth < self.locals.scope_depth => break,
                _ => {
                    if local_name == &span {
                        self.errors.sync(
                            &name.co_locate(CompileError::VariableRedeclaration(span.clone())),
                        )
                    }
                }
            }
        }
        self.add_local(name);
    }

    fn parse_variable(&mut self, error: CompileError) -> Result<(u8, Coords), ()> {
        if let Some(identifier) = self.consume(TokenKind::Identifier, error) {
            let coords = identifier.coords();
            // `span` should be put inside `else`
            let span = identifier.span().into();
            self.declare_variable(identifier);
            if self.locals.scope_depth > 0 {
                Ok((0, coords))
            } else {
                Ok((self.identifier_constant(span), coords))
            }
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

    fn for_statement(&mut self, coords: Coords) {
        self.begin_scope();
        self.consume(TokenKind::LeftParen, CompileError::ExpectedControlLeftParen);

        if self.next_token_if_eq(TokenKind::Semicolon).is_some() {
        } else if self.next_token_if_eq(TokenKind::Var).is_some() {
            self.var_declaration();
        } else {
            self.expression_statement();
        }

        let mut loop_start = self.current_chunk().len();
        let exit = if self.next_token_if_eq(TokenKind::Semicolon).is_none() {
            self.expression();
            // FIXME: error not ideal
            if let Some(coords) = self
                .consume(TokenKind::Semicolon, CompileError::UnclosedStatement)
                .map(|t| t.coords())
            {
                let jump = self.emit_jump(OpCode::JumpIfFalse, coords);
                self.emit_op(OpCode::Pop, coords);
                Some((coords, jump))
            } else {
                None
            }
        } else {
            None
        };

        if self.next_token_if_eq(TokenKind::RightParen).is_none() {
            if let Some(coords) = self.peek_token().map(|t| t.coords()) {
                let body_jump = self.emit_jump(OpCode::Jump, coords);
                let increment_start = self.current_chunk().len();
                self.expression();
                self.emit_op(OpCode::Pop, coords);
                self.emit_loop(loop_start, coords);
                loop_start = increment_start;
                self.patch_jump(body_jump);
            }
            self.consume(
                TokenKind::RightParen,
                CompileError::ExpectedControlRightParen,
            );
        }

        self.statement();
        self.emit_loop(loop_start, coords);

        if let Some((coords, jump)) = exit {
            self.patch_jump(jump);
            self.emit_op(OpCode::Pop, coords);
        }

        self.end_scope(coords);
    }

    fn if_statement(&mut self, coords: Coords) {
        self.consume(TokenKind::LeftParen, CompileError::ExpectedControlLeftParen);
        self.expression();
        self.consume(
            TokenKind::RightParen,
            CompileError::ExpectedControlRightParen,
        );

        let else_branch = self.emit_jump(OpCode::JumpIfFalse, coords);
        self.statement();
        let end = self.emit_jump(OpCode::Jump, coords);
        self.patch_jump(else_branch);
        self.emit_op(OpCode::Pop, coords);

        if self.next_token_if_eq(TokenKind::Else).is_some() {
            self.statement();
        }
        self.patch_jump(end);
    }

    fn block(&mut self) {
        loop {
            if let Some(TokenKind::RightBrace) | None = self.peek_token().map(|t| t.kind()) {
                break;
            }
            self.declaration();
        }
        self.consume(TokenKind::RightBrace, CompileError::UnclosedBlock);
    }

    fn define_variable(&mut self, global: u8, coords: Coords) {
        if self.locals.scope_depth > 0 {
            self.locals.mark_initialized();
            return;
        }
        self.emit_op(OpCode::DefineGlobal, coords);
        self.emit_byte(global, coords);
    }

    fn and(&mut self, token: &AtCoords<Token>) {
        let end = self.emit_jump(OpCode::JumpIfFalse, token.coords());
        self.emit_op(OpCode::Pop, token.coords());
        self.parse_precedence(Precedence::And);
        self.patch_jump(end);
    }

    fn parse_precedence<'b>(&'b mut self, precedence: Precedence) {
        if let Some(token) = self.peek_token() {
            let can_assign = precedence <= Precedence::Assignment;
            if self.prefix_rule(&token, can_assign).is_some() {
                while let Some(token) = self.next_token_if(|t| precedence <= Self::precedence(t)) {
                    self.infix_rule(&token).unwrap();
                }

                if let Some(coords) = self
                    .next_token_if(|t| can_assign && t == TokenKind::Equal)
                    .map(|t| t.coords())
                {
                    self.expression();
                    self.errors
                        .report(&coords.locate(CompileError::InvalidAssignmentTarget).into());
                }
            } else {
                self.errors
                    .report(&token.co_locate(CompileError::ExpectedExpression).into());
            }
        } else {
            self.errors
                .sync(&AtCoordsOrEof::Eof(CompileError::ExpectedExpression));
        }
    }

    fn prefix_rule<'b>(&'b mut self, token: &AtCoords<Token<'a>>, can_assign: bool) -> Option<()> {
        match token.kind() {
            TokenKind::LeftParen => {
                self.next_token();
                self.grouping(token)
            }
            TokenKind::Minus => {
                self.next_token();
                self.unary(token)
            }
            TokenKind::Number => {
                self.next_token();
                self.number(token)
            }
            TokenKind::False => {
                self.next_token();
                self.literal(token)
            }
            TokenKind::True => {
                self.next_token();
                self.literal(token)
            }
            TokenKind::Nil => {
                self.next_token();
                self.literal(token)
            }
            TokenKind::Bang => {
                self.next_token();
                self.unary(token)
            }
            TokenKind::String => {
                self.next_token();
                self.string(token)
            }
            TokenKind::Identifier => {
                self.next_token();
                self.variable(token, can_assign)
            }
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
            TokenKind::And => self.and(token),
            TokenKind::Or => self.or(token),
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

    fn or(&mut self, token: &AtCoords<Token<'_>>) {
        let else_branch = self.emit_jump(OpCode::JumpIfFalse, token.coords());
        let end = self.emit_jump(OpCode::Jump, token.coords());
        self.patch_jump(else_branch);
        self.emit_op(OpCode::Pop, token.coords());
        self.parse_precedence(Precedence::Or);
        self.patch_jump(end);
    }

    fn string(&mut self, token: &AtCoords<Token<'_>>) {
        self.emit_constant(
            Object::String(token.span().to_owned().into()).into(),
            token.coords(),
        )
    }

    fn resolve_local(&mut self, name: &AtCoords<Token<'_>>) -> Result<u8, ()> {
        self.locals.resolve(name.span()).map(|p| match p {
            Some(p) => p as u8,
            None => {
                self.errors.sync(&name.co_locate(
                CompileError::SelfReferencialVariableInitializer(name.span().into()),
            ));
                // error anyway
                0}
        })
    }

    fn named_variable(&mut self, token: &AtCoords<Token<'_>>, can_assign: bool) {
        let (arg, get, set) = match self.resolve_local(token) {
            Ok(arg) => (arg, OpCode::GetLocal, OpCode::SetLocal),
            Err(()) => (
                self.identifier_constant(token.span().into()),
                OpCode::GetGlobal,
                OpCode::SetGlobal,
            ),
        };

        match self
            .next_token_if(|t| can_assign && t == TokenKind::Equal)
            .map(|t| t.coords())
        {
            Some(coords) => {
                self.expression();
                self.emit_op(set, coords);
                self.emit_byte(arg, coords);
            }
            _ => {
                self.emit_op(get, token.coords());
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
