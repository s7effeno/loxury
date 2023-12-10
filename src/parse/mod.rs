mod expr;
mod stmt;
use std::iter::Peekable;

use crate::error::Syntax as SyntaxError;
use crate::lex::{Lexer, Token};
use crate::Local;
use expr::{Expr, Literal};
use stmt::Stmt;

pub struct Parser<'a> {
    tokens: Peekable<Lexer<'a>>,
    errors: Vec<Local<SyntaxError>>,
}

impl Parser<'_> {
    fn error(&mut self, error: Local<SyntaxError>) {
        self.errors.push(error);
    }

    // TODO: find better alternative to these three methods
    fn next_token(&mut self) -> Option<Local<Token>> {
        match self.tokens.next() {
            Some(l) => match l.value() {
                Ok(t) => Some(Local::new(l.row(), l.col(), t.clone())),
                Err(e) => {
                    self.error(Local::new(l.row(), l.col(), e.clone()));
                    self.next_token()
                }
            },
            None => None,
        }
    }

    fn peek_token(&mut self) -> Option<Local<Token>> {
        let peek = self.tokens.peek().cloned();
        match peek {
            Some(l) => match l.value() {
                Ok(t) => Some(Local::new(l.row(), l.col(), t.clone())),
                Err(e) => {
                    let l = l.clone();
                    self.error(Local::new(l.row(), l.col(), e.clone()));
                    self.tokens.next();
                    self.peek_token()
                }
            },
            None => None,
        }
    }

    fn next_token_if(&mut self, func: impl FnOnce(&Local<Token>) -> bool) -> Option<Local<Token>> {
        self.peek_token().filter(|c| func(c)).map(|c| {
            self.next();
            c
        })
    }

    fn statement(&mut self) -> Stmt {
        self.next_token_if(|t| matches!(t.value(), Token::Print))
            .map(|_| self.print_statement());
        self.expression_statement()
    }

    fn print_statement(&mut self) -> Stmt {
        let value = self.expression();
        self.next_token_if(|t| matches!(t.value(), Token::Semicolon));
        Stmt::Print(value)
    }

    fn expression_statement(&mut self) -> Stmt {
        let expr = self.expression();
        self.next_token_if(|t| matches!(t.value(), Token::Semicolon));
        Stmt::Print(expr)
    }

    fn expression(&mut self) -> Expr {
        self.equality()
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();

        while let Some(operator @ Token::BangEqual) | Some(operator @ Token::EqualEqual) =
            self.peek_token().map(|t| t.value().clone())
        {
            let operator = operator.clone();
            self.tokens.next();
            let right = self.comparison();
            expr = Expr::Binary(Box::new(expr), operator.clone(), Box::new(right))
        }

        expr
    }

    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();

        while let Some(operator @ Token::Greater)
        | Some(operator @ Token::GreaterEqual)
        | Some(operator @ Token::Less)
        | Some(operator @ Token::LessEqual) = self.peek_token().map(|t| t.value().clone())
        {
            let operator = operator.clone();
            self.tokens.next();
            let right = self.term();
            expr = Expr::Binary(Box::new(expr), operator.clone(), Box::new(right))
        }

        expr
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.factor();

        while let Some(operator @ Token::Minus) | Some(operator @ Token::Plus) =
            self.peek_token().map(|t| t.value().clone())
        {
            let operator = operator.clone();
            self.tokens.next();
            let right = self.factor();
            expr = Expr::Binary(Box::new(expr), operator.clone(), Box::new(right))
        }

        expr
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();

        while let Some(operator @ Token::Slash) | Some(operator @ Token::Star) =
            self.peek_token().map(|t| t.value().clone())
        {
            let operator = operator.clone();
            self.tokens.next();
            let right = self.unary();
            expr = Expr::Binary(Box::new(expr), operator.clone(), Box::new(right))
        }

        expr
    }

    fn unary(&mut self) -> Expr {
        if let Some(operator @ Token::Bang) | Some(operator @ Token::Minus) =
            self.peek_token().map(|t| t.value().clone())
        {
            let operator = operator.clone();
            self.tokens.next();
            let right = self.unary();
            Expr::Unary(operator, Box::new(right))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Expr {
        if let Some(token) = self.peek_token().map(|t| t.value().clone()) {
            self.next_token();
            match token {
                Token::False => Expr::Literal(Some(Literal::Boolean(false))),
                Token::True => Expr::Literal(Some(Literal::Boolean(true))),
                Token::Number(n) => Expr::Literal(Some(Literal::Number(n))),
                Token::String(s) => Expr::Literal(Some(Literal::String(s))),
                Token::LeftParen => {
                    let expr = self.expression();
                    if matches!(
                        self.peek_token().map(|t| t.value().clone()),
                        Some(Token::RightParen)
                    ) {
                        Expr::Grouping(Box::new(expr))
                    } else {
                        panic!()
                    }
                }
                _ => todo!(),
            }
        } else {
            panic!()
        }
    }

    fn synchronyze(&mut self) {
        /*while let Some((_, _, r)) = self.tokens.peek() {
            if let Ok(t) = r {
                if let Token::Class
                | Token::Fun
                | Token::Var
                | Token::For
                | Token::If
                | Token::While
                | Token::Print
                | Token::Return = t
                {
                    break;
                }
            }
        } */
    }
}

impl Iterator for Parser<'_> {
    type Item = Stmt;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(_) = self.tokens.peek() {
            Some(self.statement())
        } else {
            None
        }
    }
}
