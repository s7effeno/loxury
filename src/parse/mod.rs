mod expr;
mod stmt;
use std::iter::Peekable;

use crate::lex::{Lexer, Token};
use expr::{Expr, Literal};
use stmt::Stmt;

pub struct Parser<'a> {
    tokens: Peekable<Lexer<'a>>,
}

impl Parser<'_> {
    fn statement(&mut self) -> Stmt {
        self.tokens
            .next_if(|(_, _, t)| matches!(t, Ok(Token::Print)))
            .map(|_| self.print_statement());
        self.expression_statement()
    }

    fn print_statement(&mut self) -> Stmt {
        let value = self.expression();
        self.tokens
            .next_if(|(_, _, t)| matches!(t, Ok(Token::Semicolon)));
        Stmt::Print(value)
    }

    fn expression_statement(&mut self) -> Stmt {
        let expr = self.expression();
        self.tokens
            .next_if(|(_, _, t)| matches!(t, Ok(Token::Semicolon)));
        Stmt::Print(expr)
    }

    fn expression(&mut self) -> Expr {
        self.equality()
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();

        while let Some((_, _, Ok(operator @ Token::BangEqual)))
        | Some((_, _, Ok(operator @ Token::EqualEqual))) = self.tokens.peek()
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

        while let Some((_, _, Ok(operator @ Token::Greater)))
        | Some((_, _, Ok(operator @ Token::GreaterEqual)))
        | Some((_, _, Ok(operator @ Token::Less)))
        | Some((_, _, Ok(operator @ Token::LessEqual))) = self.tokens.peek()
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

        while let Some((_, _, Ok(operator @ Token::Minus)))
        | Some((_, _, Ok(operator @ Token::Plus))) = self.tokens.peek()
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

        while let Some((_, _, Ok(operator @ Token::Slash)))
        | Some((_, _, Ok(operator @ Token::Star))) = self.tokens.peek()
        {
            let operator = operator.clone();
            self.tokens.next();
            let right = self.unary();
            expr = Expr::Binary(Box::new(expr), operator.clone(), Box::new(right))
        }

        expr
    }

    fn unary(&mut self) -> Expr {
        if let Some((_, _, Ok(operator @ Token::Bang))) | Some((_, _, Ok(operator @ Token::Minus))) = self.tokens.peek() {
            let operator = operator.clone();
            self.tokens.next();
            let right = self.unary();
            Expr::Unary(operator, Box::new(right))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Expr {
        if let Some((_, _, Ok(token))) = self.tokens.peek() {
            let token = token.clone();
            self.tokens.next();
            match token {
                Token::False => Expr::Literal(Some(Literal::Boolean(false))),
                Token::True => Expr::Literal(Some(Literal::Boolean(true))),
                Token::Number(n) => Expr::Literal(Some(Literal::Number(n))),
                Token::String(s) => Expr::Literal(Some(Literal::String(s))),
                Token::LeftParen => {
                    let expr = self.expression();
                    if let Some((_, _, Ok(Token::RightParen))) = self.tokens.peek() {
                        Expr::Grouping(Box::new(expr))
                    } else {
                        panic!()
                    }
                }
                _ => unreachable!()
            }
        } else {
            panic!()
        }
    }

    fn synchronyze(&mut self) {
        while let Some((_, _, r)) = self.tokens.peek() {
            if let Ok(t) = r {
                if let Token::Class | Token::Fun | Token::Var | Token::For | Token::If | Token::While | Token::Print | Token::Return = t{
                    break;
                }
            }
        }
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
