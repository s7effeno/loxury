mod expr;
mod stmt;
use std::iter::Peekable;

use crate::error::Syntax as SyntaxError;
use crate::lex::{Lexer, Token};
use crate::Located;
pub use expr::{Expr, Literal};
pub use stmt::Stmt;

pub struct Parser<'a> {
    tokens: Peekable<Lexer<'a>>,
    errors: Vec<Located<SyntaxError>>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Lexer<'a>) -> Self {
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

    fn statement(&mut self) -> Result<Stmt, Located<SyntaxError>> {
        self.next_token_if(|t| matches!(t.value(), Token::Print))
            .map(|_| self.print_statement())
            .unwrap_or_else(|| self.expression_statement())
    }

    fn print_statement(&mut self) -> Result<Stmt, Located<SyntaxError>> {
        let value = self.expression()?;
        match self.peek_token() {
            Some(t) => self
                .next_token_if(|t| matches!(t.value(), Token::Semicolon))
                .map(|_| Ok(Stmt::Print(value)))
                .unwrap_or_else(|| Err(t.co_locate(SyntaxError::UnterminatedExprStatement))),
            None => Err(Located::at_eof(SyntaxError::UnterminatedExprStatement)),
        }
    }

    fn expression_statement(&mut self) -> Result<Stmt, Located<SyntaxError>> {
        let expr = self.expression()?;
        match self.peek_token() {
            Some(t) => self
                .next_token_if(|t| matches!(t.value(), Token::Semicolon))
                .map(|_| Ok(Stmt::Expression(expr)))
                .unwrap_or_else(|| Err(t.co_locate(SyntaxError::UnterminatedExprStatement))),
            None => Err(Located::at_eof(SyntaxError::UnterminatedExprStatement)),
        }
    }

    fn expression(&mut self) -> Result<Expr, Located<SyntaxError>> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, Located<SyntaxError>> {
        let mut expr = self.comparison()?;

        while let Some(t) = self.peek_token() {
            if let Token::BangEqual | Token::EqualEqual = t.value() {
                self.tokens.next();
                let right = self.comparison()?;
                expr = Expr::Binary(Box::new(expr), t.clone(), Box::new(right))
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, Located<SyntaxError>> {
        let mut expr = self.term()?;

        while let Some(t) = self.peek_token() {
            if let Token::Greater | Token::GreaterEqual | Token::Less | Token::LessEqual = t.value()
            {
                self.tokens.next();
                let right = self.term()?;
                expr = Expr::Binary(Box::new(expr), t.clone(), Box::new(right))
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, Located<SyntaxError>> {
        let mut expr = self.factor()?;

        while let Some(t) = self.peek_token() {
            if let Token::Minus | Token::Plus = t.value() {
                self.tokens.next();
                let right = self.factor()?;
                expr = Expr::Binary(Box::new(expr), t.clone(), Box::new(right))
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, Located<SyntaxError>> {
        let mut expr = self.unary()?;

        while let Some(t) = self.peek_token() {
            if let Token::Slash | Token::Star = t.value() {
                self.tokens.next();
                let right = self.unary()?;
                expr = Expr::Binary(Box::new(expr), t.clone(), Box::new(right))
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, Located<SyntaxError>> {
        if let Some(t) = self.peek_token() {
            if let Token::Bang | Token::Minus = t.value() {
                self.tokens.next();
                let right = self.unary()?;
                return Ok(Expr::Unary(t.clone(), Box::new(right)));
            }
        }

        return self.primary();
    }

    fn primary(&mut self) -> Result<Expr, Located<SyntaxError>> {
        if let Some(token) = self.peek_token() {
            match token.value() {
                Token::False => {
                    self.tokens.next();
                    Ok(Expr::Literal(Literal::Boolean(false)))
                }
                Token::True => {
                    self.tokens.next();
                    Ok(Expr::Literal(Literal::Boolean(true)))
                }
                Token::Nil => {
                    self.tokens.next();
                    Ok(Expr::Literal(Literal::Nil))
                }
                Token::Number(n) => {
                    self.tokens.next();
                    Ok(Expr::Literal(Literal::Number(*n)))
                }
                Token::String(s) => {
                    self.tokens.next();
                    Ok(Expr::Literal(Literal::String(s.to_owned())))
                }
                Token::LeftParen => {
                    self.tokens.next();
                    let expr = self.expression()?;
                    match self.peek_token() {
                        Some(t) => self
                            .next_token_if(|t| matches!(t.value(), Token::RightParen))
                            .map(|_| Ok(Expr::Grouping(Box::new(expr))))
                            .unwrap_or_else(|| Err(t.co_locate(SyntaxError::UnclosedGrouping))),
                        None => Err(Located::at_eof(SyntaxError::UnclosedGrouping)),
                    }
                }
                _ => Err(token.co_locate(SyntaxError::ExpectedExpression)),
            }
        } else {
            Err(Located::at_eof(SyntaxError::ExpectedExpression))
        }
    }

    fn synchronyze(&mut self) {
        while let Some(t) = self.peek_token() {
            match t.value() {
                Token::Class
                | Token::Fun
                | Token::Var
                | Token::For
                | Token::If
                | Token::While
                | Token::Print
                | Token::Return => {
                    break;
                }
                Token::Semicolon => {
                    self.tokens.next();
                    break;
                }
                _ => {
                    self.tokens.next();
                }
            }
        }
    }
}

impl Iterator for Parser<'_> {
    type Item = Stmt;

    fn next(&mut self) -> Option<Self::Item> {
        match self.peek_token() {
            Some(_) => match self.statement() {
                Ok(stmt) => Some(stmt),
                Err(e) => {
                    self.error(e);
                    self.synchronyze();
                    self.next()
                }
            },
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asd() {
        let mut p = Parser::new(Lexer::new("print 6asdfsadf;"));
        println!("{:?}", p.next());
        println!("{:?}", p.errors,);
        p.next();
        println!("{:?}", p.errors,);
    }
}
