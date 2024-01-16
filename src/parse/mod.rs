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

    fn declaration(&mut self) -> Result<Stmt, Located<SyntaxError>> {
        self.next_token_if(|t| matches!(t.value(), Token::Var))
            .map(|_| self.var_declaration())
            .unwrap_or_else(|| self.statement())
    }

    // TODO: write this shit better
    fn var_declaration(&mut self) -> Result<Stmt, Located<SyntaxError>> {
        if let Some(t) = self.peek_token() {
            if let Token::Identifier(i) = t.value() {
                self.tokens.next();

                let initializer = if let Some(t) = self.peek_token() {
                    if let Token::Equal = t.value() {
                        self.tokens.next();
                        Some(self.expression()?)
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some(t) = self.peek_token() {
                    if let Token::Semicolon = t.value() {
                        self.tokens.next();
                        Ok(Stmt::Var(t.co_locate(i.to_owned()), initializer))
                    } else {
                        Err(t.co_locate(SyntaxError::UnterminatedExprStatement))
                    }
                } else {
                    Err(Located::at_eof(SyntaxError::UnterminatedExprStatement))
                }
            } else {
                Err(t.co_locate(SyntaxError::ExpectedVariableName))
            }
        } else {
            Err(Located::at_eof(SyntaxError::ExpectedVariableName))
        }
    }

    fn statement(&mut self) -> Result<Stmt, Located<SyntaxError>> {
        match self.peek_token() {
            Some(t) => match t.value() {
                Token::For => {
                    self.tokens.next();
                    self.for_statement()
                }
                Token::While => {
                    self.tokens.next();
                    self.while_statement()
                }
                Token::If => {
                    self.tokens.next();
                    self.if_statement()
                }
                Token::Print => {
                    self.tokens.next();
                    self.print_statement()
                }
                Token::LeftBrace => {
                    self.tokens.next();
                    Ok(Stmt::Block(self.block()?))
                }
                _ => self.expression_statement(),
            },
            None => self.expression_statement(),
        }
    }

    fn for_statement(&mut self) -> Result<Stmt, Located<SyntaxError>> {
        let Some(t) = self.peek_token() else {
            return Err(Located::at_eof(SyntaxError::ExpectedControlLeftParen));
        };
        let Token::LeftParen = t.value() else {
            return Err(t.co_locate(SyntaxError::ExpectedControlLeftParen));
        };
        self.tokens.next();

        let Some(t) = self.peek_token() else {
            return Err(Located::at_eof(SyntaxError::ExpectedSemiColonAfterForInit));
        };
        let initializer = match t.value() {
            Token::Semicolon => {
                self.tokens.next();
                None
            }
            Token::Var => {
                self.tokens.next();
                Some(self.var_declaration()?)
            }
            _ => Some(self.expression_statement()?),
        };
        println!("initializer: {:?}", initializer);

        let Some(t) = self.peek_token() else {
            return Err(Located::at_eof(
                SyntaxError::ExpectedSemicolonAfterForCondition,
            ));
        };
        let condition = if let Token::Semicolon = t.value() {
            self.tokens.next();
            None
        } else {
            Some(self.expression()?)
        };
        if let Some(t) = self.peek_token() {
            let Token::Semicolon = t.value() else {
                return Err(t.co_locate(SyntaxError::ExpectedSemicolonAfterForCondition));
            };
            self.tokens.next();
        }
        println!("condition: {:?}", condition);

        let Some(t) = self.peek_token() else {
            return Err(Located::at_eof(SyntaxError::ExpectedControlRightParen));
        };
        let increment = if let Token::RightParen = t.value() {
            None
        } else {
            Some(self.expression()?)
        };
        let Some(t) = self.peek_token() else {
            return Err(Located::at_eof(SyntaxError::ExpectedControlRightParen));
        };
        let Token::RightParen = t.value() else {
            return Err(t.co_locate(SyntaxError::ExpectedControlRightParen));
        };
        self.tokens.next();

        let mut body = self.statement()?;

        if let Some(i) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(i)]);
        }

        body = Stmt::While(
            if let Some(c) = condition {
                c
            } else {
                Expr::Literal(Literal::Boolean(true))
            },
            Box::new(body),
        );

        if let Some(i) = initializer {
            body = Stmt::Block(vec![i, body])
        }
        println!("{:?}", body);

        Ok(body)
    }

    fn while_statement(&mut self) -> Result<Stmt, Located<SyntaxError>> {
        let condition = if let Some(t) = self.peek_token() {
            if let Token::LeftParen = t.value() {
                self.tokens.next();
                self.expression()
            } else {
                Err(t.co_locate(SyntaxError::ExpectedControlLeftParen))
            }
        } else {
            Err(Located::at_eof(SyntaxError::ExpectedControlLeftParen))
        }?;
        if let Some(t) = self.peek_token() {
            if let Token::RightParen = t.value() {
                self.tokens.next();
                Ok(())
            } else {
                Err(t.co_locate(SyntaxError::ExpectedControlRightParen))
            }
        } else {
            Err(Located::at_eof(SyntaxError::ExpectedControlRightParen))
        }?;
        let body = self.statement()?;

        Ok(Stmt::While(condition, Box::new(body)))
    }

    fn if_statement(&mut self) -> Result<Stmt, Located<SyntaxError>> {
        let condition = if let Some(t) = self.peek_token() {
            if let Token::LeftParen = t.value() {
                self.tokens.next();
                self.expression()
            } else {
                Err(t.co_locate(SyntaxError::ExpectedControlLeftParen))
            }
        } else {
            Err(Located::at_eof(SyntaxError::ExpectedControlLeftParen))
        }?;
        if let Some(t) = self.peek_token() {
            if let Token::RightParen = t.value() {
                self.tokens.next();
                Ok(())
            } else {
                Err(t.co_locate(SyntaxError::ExpectedControlRightParen))
            }
        } else {
            Err(Located::at_eof(SyntaxError::ExpectedControlRightParen))
        }?;
        let branch_then = self.statement()?;
        let branch_else = if let Some(t) = self.peek_token() {
            if let Token::Else = t.value() {
                self.tokens.next();
                Some(self.statement()?)
            } else {
                None
            }
        } else {
            None
        };
        Ok(Stmt::If(
            condition,
            Box::new(branch_then),
            branch_else.map(Box::new),
        ))
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

    fn block(&mut self) -> Result<Vec<Stmt>, Located<SyntaxError>> {
        let mut statements = Vec::new();
        while let Some(t) = self.peek_token() {
            if let Token::RightBrace = t.value() {
                break;
            }
            statements.push(self.declaration()?);
        }
        if let Some(t) = self.peek_token() {
            if let Token::RightBrace = t.value() {
                self.tokens.next();
                Ok(statements)
            } else {
                Err(t.co_locate(SyntaxError::UnterminatedBlock))
            }
        } else {
            Err(Located::at_eof(SyntaxError::UnterminatedBlock))
        }
    }

    fn expression(&mut self) -> Result<Expr, Located<SyntaxError>> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, Located<SyntaxError>> {
        let expr = self.or()?;

        if let Some(t) = self.next_token_if(|t| matches!(t.value(), Token::Equal)) {
            let value = self.assignment()?;
            if let Expr::Variable(name) = expr {
                Ok(Expr::Assign(name, Box::new(value)))
            } else {
                Err(t.co_locate(SyntaxError::InvalidAssignmentTarget))
            }
        } else {
            Ok(expr)
        }
    }

    fn or(&mut self) -> Result<Expr, Located<SyntaxError>> {
        let mut expr = self.and()?;

        while let Some(t) = self.peek_token() {
            if let Token::Or = t.value() {
                let operator = t.clone();
                self.tokens.next();
                let right = self.and()?;
                expr = Expr::Logical(Box::new(expr), operator, Box::new(right));
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, Located<SyntaxError>> {
        let mut expr = self.equality()?;

        while let Some(t) = self.peek_token() {
            if let Token::And = t.value() {
                let operator = t.clone();
                self.tokens.next();
                let right = self.equality()?;
                expr = Expr::Logical(Box::new(expr), operator, Box::new(right));
            } else {
                break;
            }
        }

        Ok(expr)
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

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, Located<SyntaxError>> {
        if let Some(t) = self.peek_token() {
            match t.value() {
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
                Token::Identifier(i) => {
                    self.tokens.next();
                    Ok(Expr::Variable(t.co_locate(i.to_owned())))
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
                _ => Err(t.co_locate(SyntaxError::ExpectedExpression)),
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
            Some(_) => match self.declaration() {
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
    fn aaa() {
        let mut p = Parser::new(Lexer::new(
            "
            var a = 0;
            var temp;

            for (var b = 1; a < 10000; b = temp + b) {
              print a;
              temp = a;
              a = b;
            }",
        ));
        println!("{:#?}", p.next());
        println!("{:#?}", p.next());
        println!("{:#?}", p.next());
        println!("{:?}", p.errors);
    }
}
