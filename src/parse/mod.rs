// TODO: implement better way to handle Identifiers
//       improve next_token_if_or_err
mod expr;
mod stmt;
use std::iter::Peekable;

use crate::error::Syntax as SyntaxError;
use crate::lex::{Lexer, Token};
use crate::Located;
pub use expr::{Expr, Literal};
pub use stmt::{Function, Stmt};

use std::rc::Rc;

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

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ()> {
        let statements = self.collect();
        if self.errors.len() > 0 {
            Err(())
        } else {
            Ok(statements)
        }
    }

    pub fn errors(&self) -> &[Located<SyntaxError>] {
        &self.errors
    }

    fn error(&mut self, error: Located<SyntaxError>) {
        self.errors.push(error);
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

    fn next_token_if(&mut self, func: impl FnOnce(&Token) -> bool) -> Option<Located<Token>> {
        self.peek_token().filter(|c| func(c.value())).map(|c| {
            self.tokens.next();
            c
        })
    }

    fn next_token_if_or_err(
        &mut self,
        func: impl FnOnce(&Token) -> bool,
    ) -> Result<Located<Token>, Located<()>> {
        let token = self
            .peek_token()
            .ok_or(())
            .map_err(|_| Located::at_eof(()))?;
        if func(token.value()) {
            self.tokens.next();
            Ok(token)
        } else {
            Err(token.co_locate(()))
        }
    }

    fn declaration(&mut self) -> Result<Stmt, Located<SyntaxError>> {
        if self.next_token_if(|t| matches!(t, Token::Class)).is_some() {
            self.class_declaration()
        } else if self.next_token_if(|t| matches!(t, Token::Fun)).is_some() {
            Ok(Stmt::Function(self.function()?))
        } else if self.next_token_if(|t| matches!(t, Token::Var)).is_some() {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn class_declaration(&mut self) -> Result<Stmt, Located<SyntaxError>> {
        let t = self
            .next_token_if_or_err(|t| matches!(t, Token::Identifier(_)))
            .map_err(|e| e.co_locate(SyntaxError::ExpectedClassName))?;
        let Token::Identifier(name) = t.value() else {
            unreachable!()
        };
        let name = t.co_locate(name.to_owned());

        let superclass = if self.next_token_if(|t| matches!(t, Token::Less)).is_some() {
            let t = self
                .next_token_if_or_err(|t| matches!(t, Token::Identifier(_)))
                .map_err(|e| e.co_locate(SyntaxError::ExpectedSuperClassName))?;
            let Token::Identifier(superclass) = t.value() else {
                unreachable!()
            };
            Some(Expr::Variable(t.co_locate(superclass.into())))
        } else {
            None
        };

        self.next_token_if_or_err(|t| matches!(t, Token::LeftBrace))
            .map_err(|e| e.co_locate(SyntaxError::UnopenedBlock))?;

        let mut methods = Vec::new();
        while self
            .peek_token()
            .is_some_and(|t| !matches!(t.value(), Token::RightBrace))
        {
            methods.push(self.function()?);
        }

        self.next_token_if_or_err(|t| matches!(t, Token::RightBrace))
            .map_err(|e| e.co_locate(SyntaxError::UnclosedBlock))?;

        Ok(Stmt::Class(name, superclass, methods))
    }

    fn var_declaration(&mut self) -> Result<Stmt, Located<SyntaxError>> {
        let t = self
            .next_token_if_or_err(|t| matches!(t, Token::Identifier(_)))
            .map_err(|e| e.co_locate(SyntaxError::ExpectedVariableName))?;
        let Token::Identifier(identifier) = t.value() else {
            unreachable!()
        };

        let initializer = if let Some(_) = self.next_token_if(|t| matches!(t, Token::Equal)) {
            Some(self.expression()?)
        } else {
            None
        };

        self.next_token_if_or_err(|t| matches!(t, Token::Semicolon))
            .map_err(|e| e.co_locate(SyntaxError::UnclosedStatement))?;

        Ok(Stmt::Var(t.co_locate(identifier.to_owned()), initializer))
    }

    fn statement(&mut self) -> Result<Stmt, Located<SyntaxError>> {
        if self.next_token_if(|t| matches!(t, Token::For)).is_some() {
            self.for_statement()
        } else if self.next_token_if(|t| matches!(t, Token::While)).is_some() {
            self.while_statement()
        } else if self.next_token_if(|t| matches!(t, Token::If)).is_some() {
            self.if_statement()
        } else if self.next_token_if(|t| matches!(t, Token::Print)).is_some() {
            self.print_statement()
        } else if let Some(t) = self.next_token_if(|t| matches!(t, Token::Return)) {
            self.return_statement(t.co_locate(()))
        } else if self
            .next_token_if(|t| matches!(t, Token::LeftBrace))
            .is_some()
        {
            Ok(Stmt::Block(self.block()?))
        } else {
            self.expression_statement()
        }
    }

    fn for_statement(&mut self) -> Result<Stmt, Located<SyntaxError>> {
        self.next_token_if_or_err(|t| matches!(t, Token::LeftParen))
            .map_err(|e| e.co_locate(SyntaxError::ExpectedControlLeftParen))?;

        let initializer = if self
            .next_token_if(|t| matches!(t, Token::Semicolon))
            .is_some()
        {
            None
        } else if self.next_token_if(|t| matches!(t, Token::Var)).is_some() {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if self
            .peek_token()
            .is_some_and(|t| matches!(t.value(), Token::Semicolon))
        {
            None
        } else {
            Some(self.expression()?)
        };
        self.next_token_if_or_err(|t| matches!(t, Token::Semicolon))
            .map_err(|e| e.co_locate(SyntaxError::ExpectedSemicolonAfterForCondition))?;

        let increment = if self
            .peek_token()
            .is_some_and(|t| matches!(t.value(), Token::RightParen))
        {
            None
        } else {
            Some(self.expression()?)
        };
        self.next_token_if_or_err(|t| matches!(t, Token::RightParen))
            .map_err(|e| e.co_locate(SyntaxError::ExpectedControlRightParen))?;

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

        Ok(body)
    }

    fn while_statement(&mut self) -> Result<Stmt, Located<SyntaxError>> {
        self.next_token_if_or_err(|t| matches!(t, Token::LeftParen))
            .map_err(|e| e.co_locate(SyntaxError::ExpectedControlLeftParen))?;

        let condition = self.expression()?;

        self.next_token_if_or_err(|t| matches!(t, Token::RightParen))
            .map_err(|e| e.co_locate(SyntaxError::ExpectedControlRightParen))?;

        let body = self.statement()?;

        Ok(Stmt::While(condition, Box::new(body)))
    }

    fn if_statement(&mut self) -> Result<Stmt, Located<SyntaxError>> {
        self.next_token_if_or_err(|t| matches!(t, Token::LeftParen))
            .map_err(|e| e.co_locate(SyntaxError::ExpectedControlLeftParen))?;

        let condition = self.expression()?;

        self.next_token_if_or_err(|t| matches!(t, Token::RightParen))
            .map_err(|e| e.co_locate(SyntaxError::ExpectedControlRightParen))?;

        let branch_then = self.statement()?;

        let branch_else = if self.next_token_if(|t| matches!(t, Token::Else)).is_some() {
            Some(self.statement()?)
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

        self.next_token_if_or_err(|t| matches!(t, Token::Semicolon))
            .and(Ok(Stmt::Print(value)))
            .map_err(|e| e.co_locate(SyntaxError::UnclosedStatement))
    }

    fn return_statement(&mut self, location: Located<()>) -> Result<Stmt, Located<SyntaxError>> {
        let value = if !self
            .peek_token()
            .is_some_and(|t| matches!(t.value(), Token::Semicolon))
        {
            self.expression()?
        } else {
            Expr::Literal(Literal::Nil)
        };

        self.next_token_if_or_err(|t| matches!(t, Token::Semicolon))
            .map_err(|e| e.co_locate(SyntaxError::UnclosedStatement))?;

        Ok(Stmt::Return(location, value))
    }

    fn expression_statement(&mut self) -> Result<Stmt, Located<SyntaxError>> {
        let expr = self.expression()?;

        self.next_token_if_or_err(|t| matches!(t, Token::Semicolon))
            .and(Ok(Stmt::Expression(expr)))
            .map_err(|e| e.co_locate(SyntaxError::UnclosedStatement))
    }

    fn function(&mut self) -> Result<Rc<Function>, Located<SyntaxError>> {
        let t = self
            .next_token_if_or_err(|t| matches!(t, Token::Identifier(_)))
            .map_err(|e| e.co_locate(SyntaxError::ExpectedFunctionName))?;
        let Token::Identifier(name) = t.value() else {
            unreachable!()
        };
        let name = t.co_locate(name.to_owned());

        self.next_token_if_or_err(|t| matches!(t, Token::LeftParen))
            .map_err(|e| e.co_locate(SyntaxError::ExpectedFunctionLeftParen))?;

        let mut params = Vec::new();

        if self
            .peek_token()
            .is_some_and(|t| !matches!(t.value(), Token::RightParen))
        {
            loop {
                let t = self
                    .next_token_if_or_err(|t| matches!(t, Token::Identifier(_)))
                    .map_err(|e| e.co_locate(SyntaxError::ExpectedParameterName))?;
                let Token::Identifier(name) = t.value() else {
                    unreachable!()
                };
                params.push(t.co_locate(name.to_owned()));
                if self.next_token_if(|t| matches!(t, Token::Comma)).is_none() {
                    break;
                }
            }
        }

        self.next_token_if_or_err(|t| matches!(t, Token::RightParen))
            .map_err(|e| e.co_locate(SyntaxError::ExpectedControlRightParen))?;

        self.next_token_if_or_err(|t| matches!(t, Token::LeftBrace))
            .map_err(|e| e.co_locate(SyntaxError::UnopenedBlock))?;

        let body = self.block()?;
        Ok(Function { name, body, params }.into())
    }

    fn block(&mut self) -> Result<Vec<Stmt>, Located<SyntaxError>> {
        let mut statements = Vec::new();
        while self
            .peek_token()
            .is_some_and(|t| !matches!(t.value(), Token::RightBrace))
        {
            statements.push(self.declaration()?);
        }

        self.next_token_if_or_err(|t| matches!(t, Token::RightBrace))
            .map_err(|e| e.co_locate(SyntaxError::UnclosedBlock))?;

        Ok(statements)
    }

    fn expression(&mut self) -> Result<Expr, Located<SyntaxError>> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, Located<SyntaxError>> {
        let expr = self.or()?;

        if let Some(t) = self.next_token_if(|t| matches!(t, Token::Equal)) {
            let value = self.assignment()?;
            match expr {
                Expr::Variable(name) => Ok(Expr::Assign(name, value.into())),
                Expr::Get(object, name) => Ok(Expr::Set(object, name, value.into())),
                _ => {
                    self.error(t.co_locate(SyntaxError::InvalidAssignmentTarget));
                    Ok(expr)
                }
            }
        } else {
            Ok(expr)
        }
    }

    fn or(&mut self) -> Result<Expr, Located<SyntaxError>> {
        let mut expr = self.and()?;

        while let Some(t) = self.next_token_if(|t| matches!(t, Token::Or)) {
            let operator = t.clone();
            let right = self.and()?;
            expr = Expr::Logical(expr.into(), operator, right.into());
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, Located<SyntaxError>> {
        let mut expr = self.equality()?;

        while let Some(t) = self.next_token_if(|t| matches!(t, Token::And)) {
            let operator = t.clone();
            let right = self.equality()?;
            expr = Expr::Logical(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, Located<SyntaxError>> {
        let mut expr = self.comparison()?;

        while let Some(t) = self.next_token_if(|t| matches!(t, Token::Bang | Token::BangEqual)) {
            let operator = t.clone();
            let right = self.comparison()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, Located<SyntaxError>> {
        let mut expr = self.term()?;

        while let Some(t) = self.next_token_if(|t| {
            matches!(
                t,
                Token::EqualEqual
                    | Token::Greater
                    | Token::GreaterEqual
                    | Token::Less
                    | Token::LessEqual
            )
        }) {
            let operator = t.clone();
            let right = self.term()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right))
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, Located<SyntaxError>> {
        let mut expr = self.factor()?;

        while let Some(t) = self.next_token_if(|t| matches!(t, Token::Minus | Token::Plus)) {
            let operator = t.clone();
            let right = self.factor()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right))
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, Located<SyntaxError>> {
        let mut expr = self.unary()?;

        while let Some(t) = self.next_token_if(|t| matches!(t, Token::Slash | Token::Star)) {
            let operator = t.clone();
            let right = self.unary()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right))
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, Located<SyntaxError>> {
        if let Some(t) = self.next_token_if(|t| matches!(t, Token::Bang | Token::Minus)) {
            let operator = t.clone();
            let right = self.unary()?;
            return Ok(Expr::Unary(operator, Box::new(right)));
        }

        self.call()
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, Located<SyntaxError>> {
        let mut arguments = Vec::new();
        if !self
            .peek_token()
            .is_some_and(|t| matches!(t.value(), Token::RightParen))
        {
            loop {
                if arguments.len() >= 255 {
                    let t = self.peek_token().unwrap();
                    self.error(t.co_locate(SyntaxError::TooManyArguments));
                }
                arguments.push(self.expression()?);
                if self.next_token_if(|t| matches!(t, Token::Comma)).is_none() {
                    break;
                }
            }
        }

        let paren = self
            .next_token_if_or_err(|t| matches!(t, Token::RightParen))
            .map_err(|e| e.co_locate(SyntaxError::UnclosedArgumentsList))?;

        Ok(Expr::Call(Box::new(callee), paren, arguments))
    }

    fn call(&mut self) -> Result<Expr, Located<SyntaxError>> {
        let mut expr = self.primary()?;

        loop {
            if self
                .next_token_if(|t| matches!(t, Token::LeftParen))
                .is_some()
            {
                expr = self.finish_call(expr)?;
            } else if self.next_token_if(|t| matches!(t, Token::Dot)).is_some() {
                let t = self
                    .next_token_if_or_err(|t| matches!(t, Token::Identifier(_)))
                    .map_err(|e| e.co_locate(SyntaxError::ExpectedPropertyName))?;
                let Token::Identifier(name) = t.value() else {
                    unreachable!()
                };
                expr = Expr::Get(expr.into(), t.co_locate(name.into()))
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, Located<SyntaxError>> {
        if self.next_token_if(|t| matches!(t, Token::False)).is_some() {
            Ok(Expr::Literal(Literal::Boolean(false)))
        } else if self.next_token_if(|t| matches!(t, Token::True)).is_some() {
            Ok(Expr::Literal(Literal::Boolean(true)))
        } else if self.next_token_if(|t| matches!(t, Token::Nil)).is_some() {
            Ok(Expr::Literal(Literal::Nil))
        } else if let Some(t) = self.next_token_if(|t| matches!(t, Token::This)) {
            Ok(Expr::This(t.co_locate(())))
        } else if let Some(t) = self.next_token_if(|t| matches!(t, Token::Super)) {
            let loc = t.co_locate(());
            self.next_token_if_or_err(|t| matches!(t, Token::Dot))
                .map_err(|e| e.co_locate(SyntaxError::ExpectedSuperClassName))?;
            let t = self
                .next_token_if_or_err(|t| matches!(t, Token::Identifier(_)))
                .map_err(|e| e.co_locate(SyntaxError::IncompleteSuper))?;
            let Token::Identifier(method) = t.value() else {
                unreachable!()
            };
            let method = t.co_locate(method.into());
            Ok(Expr::Super(loc, method))
        } else if let Some(t) = self.next_token_if(|t| matches!(t, Token::Number(_))) {
            let Token::Number(n) = t.value() else {
                unreachable!()
            };
            Ok(Expr::Literal(Literal::Number(*n)))
        } else if let Some(t) = self.next_token_if(|t| matches!(t, Token::String(_))) {
            let Token::String(s) = t.value() else {
                unreachable!()
            };
            Ok(Expr::Literal(Literal::String(s.to_owned())))
        } else if let Some(t) = self.next_token_if(|t| matches!(t, Token::Identifier(_))) {
            let Token::Identifier(i) = t.value() else {
                unreachable!()
            };
            Ok(Expr::Variable(t.co_locate(i.to_owned())))
        } else if self
            .next_token_if(|t| matches!(t, Token::LeftParen))
            .is_some()
        {
            let expr = self.expression()?;
            self.next_token_if_or_err(|t| matches!(t, Token::RightParen))
                .map_err(|e| e.co_locate(SyntaxError::UnclosedGrouping))?;
            Ok(Expr::Grouping(Box::new(expr)))
        } else {
            Err(self
                .next_token_if_or_err(|_| false)
                .map_err(|e| e.co_locate(SyntaxError::ExpectedExpression))
                .unwrap_err())
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

    #[test]
    fn bbb() {
        let mut p = Parser::new(Lexer::new(
            "
            fun add(a, b, c) {
                print a + b + c;
            }
            ",
        ));
        println!("{:#?}", p.next());
        println!("{:#?}", p.next());
    }
}
