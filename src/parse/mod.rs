mod expr;
mod stmt;
use std::iter::Peekable;

use crate::error::Syntax as SyntaxError;
use crate::lex::{Lexer, Token};
use crate::Located;
use expr::{Expr, Literal};
use stmt::Stmt;

pub struct Parser<'a> {
    tokens: Peekable<Lexer<'a>>,
    errors: Vec<Located<SyntaxError>>,
}

impl<'a> Parser<'a> {
    fn new(tokens: Lexer<'a>) -> Self {
        Self {
            tokens: tokens.peekable(),
            errors: Vec::new(),
        }
    }

    fn error(&mut self, error: Located<SyntaxError>) {
        self.errors.push(error);
    }

    // TODO: find better alternative to these three methods
    fn next_token(&mut self) -> Option<Located<Token>> {
        match self.tokens.next() {
            Some(Ok(t)) => Some(t),
            Some(Err(e)) => {
                self.error(e);
                self.next_token()
            }
            None => None,
        }
    }

    fn peek_token(&mut self) -> Option<Located<Token>> {
        let peek = self.tokens.peek().cloned();
        match peek {
            Some(Ok(t)) => Some(t),
            Some(Err(e)) => {
                self.error(e);
                self.tokens.next();
                self.peek_token()
            }
            None => None,
        }
    }

    fn next_token_if(
        &mut self,
        func: impl FnOnce(&Located<Token>) -> bool,
    ) -> Option<Located<Token>> {
        self.peek_token().filter(|c| func(c)).map(|c| {
            self.tokens.next();
            c
        })
    }

    fn statement(&mut self) -> Stmt {
        self.next_token_if(|t| matches!(t.value(), Token::Print))
            .map(|_| self.print_statement())
            .unwrap_or_else(|| self.expression_statement())
    }

    fn print_statement(&mut self) -> Stmt {
        let value = self.expression();
        self.next_token_if(|t| matches!(t.value(), Token::Semicolon));
        Stmt::Print(value)
    }

    fn expression_statement(&mut self) -> Stmt {
        let expr = self.expression();
        self.next_token_if(|t| matches!(t.value(), Token::Semicolon));
        Stmt::Expression(expr)
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
            match token {
                Token::False => {
                    self.tokens.next();
                    Expr::Literal(Some(Literal::Boolean(false)))
                }
                Token::True => {
                    self.tokens.next();
                    Expr::Literal(Some(Literal::Boolean(true)))
                }
                Token::Number(n) => {
                    self.tokens.next();
                    Expr::Literal(Some(Literal::Number(n)))
                }
                Token::String(s) => {
                    self.tokens.next();
                    Expr::Literal(Some(Literal::String(s)))
                }
                Token::LeftParen => {
                    self.tokens.next();
                    let expr = self.expression();
                    if matches!(
                        self.peek_token().map(|t| t.value().clone()),
                        Some(Token::RightParen)
                    ) {
                        self.tokens.next();
                        Expr::Grouping(Box::new(expr))
                    } else {
                        panic!(
                            "{}",
                            format!(
                                "expected ')', found {:?}",
                                self.peek_token().map(|t| t.value().clone())
                            )
                        )
                    }
                }
                _ => panic!("{}", format!("expected expression, got {:?}", token)),
            }
        } else {
            panic!("expected expression");
        }
    }

    fn synchronyze(&mut self) {
        while let Some(t) = self.peek_token().map(|t| t.value().clone()) {
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
            } else {
                self.tokens.next();
            }
        }
    }
}

impl Iterator for Parser<'_> {
    type Item = Stmt;

    fn next(&mut self) -> Option<Self::Item> {
        self.peek_token().map(|_| self.statement())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asd() {
        println!("{:?}", Parser::new(Lexer::new("print (-5 * -6);")).next());
    }
}
