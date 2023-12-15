use crate::error::Runtime as RuntimeError;
use crate::lex::Token;
use crate::parse::{Expr, Literal, Stmt};
use crate::Located;

mod environment;
use environment::Environment;

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    fn new() -> Self {
        Self {
            environment: Environment::new(),
        }
    }

    fn evaluate(&self, expr: Expr) -> Result<Literal, Located<RuntimeError>> {
        fn is_equal(left: Literal, right: Literal) -> bool {
            match (left, right) {
                (Literal::Nil, Literal::Nil) => true,
                (Literal::Boolean(left), Literal::Boolean(right)) => left == right,
                (Literal::Number(left), Literal::Number(right)) => left == right,
                (Literal::String(left), Literal::String(right)) => left == right,
                _ => false,
            }
        }

        match expr {
            Expr::Literal(e) => Ok(e),
            Expr::Grouping(e) => self.evaluate(*e),
            Expr::Unary(op, e) => {
                let right = self.evaluate(*e)?;
                match op.value() {
                    Token::Minus => {
                        if let Literal::Number(n) = right {
                            Ok(Literal::Number(-n))
                        } else {
                            panic!();
                        }
                    }
                    Token::Bang => Ok(Literal::Boolean({
                        match right {
                            Literal::Nil => false,
                            Literal::Boolean(b) => b,
                            _ => true,
                        }
                    })),
                    _ => unreachable!(),
                }
            }
            Expr::Binary(l, op, r) => {
                let left = self.evaluate(*l)?;
                let right = self.evaluate(*r)?;
                match op.value() {
                    Token::Greater => {
                        if let (Literal::Number(left), Literal::Number(right)) = (left, right) {
                            Ok(Literal::Boolean(left > right))
                        } else {
                            Err(op.co_locate(RuntimeError::ExpectedNumbers))
                        }
                    }
                    Token::GreaterEqual => {
                        if let (Literal::Number(left), Literal::Number(right)) = (left, right) {
                            Ok(Literal::Boolean(left >= right))
                        } else {
                            Err(op.co_locate(RuntimeError::ExpectedNumbers))
                        }
                    }
                    Token::Less => {
                        if let (Literal::Number(left), Literal::Number(right)) = (left, right) {
                            Ok(Literal::Boolean(left < right))
                        } else {
                            Err(op.co_locate(RuntimeError::ExpectedNumbers))
                        }
                    }
                    Token::LessEqual => {
                        if let (Literal::Number(left), Literal::Number(right)) = (left, right) {
                            Ok(Literal::Boolean(left <= right))
                        } else {
                            Err(op.co_locate(RuntimeError::ExpectedNumbers))
                        }
                    }
                    Token::Minus => {
                        if let (Literal::Number(left), Literal::Number(right)) = (left, right) {
                            Ok(Literal::Number(left - right))
                        } else {
                            Err(op.co_locate(RuntimeError::ExpectedNumbers))
                        }
                    }
                    Token::BangEqual => Ok(Literal::Boolean(!is_equal(left, right))),
                    Token::EqualEqual => Ok(Literal::Boolean(is_equal(left, right))),
                    Token::Plus => match (left, right) {
                        (Literal::Number(left), Literal::Number(right)) => {
                            Ok(Literal::Number(left + right))
                        }
                        (Literal::String(left), Literal::String(right)) => {
                            Ok(Literal::String(left + &right))
                        }
                        _ => Err(op.co_locate(RuntimeError::ExpectedNumbersOrStrings)),
                    },
                    Token::Slash => {
                        if let (Literal::Number(left), Literal::Number(right)) = (left, right) {
                            Ok(Literal::Number(left / right))
                        } else {
                            Err(op.co_locate(RuntimeError::ExpectedNumbers))
                        }
                    }
                    Token::Star => {
                        if let (Literal::Number(left), Literal::Number(right)) = (left, right) {
                            Ok(Literal::Number(left * right))
                        } else {
                            Err(op.co_locate(RuntimeError::ExpectedNumbers))
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Expr::Variable(name) => self.environment.get(name),
        }
    }

    fn execute(&mut self, stmt: Stmt) -> Result<(), Located<RuntimeError>> {
        match stmt {
            Stmt::Print(e) => {
                println!("{}", self.evaluate(e)?);
                Ok(())
            }
            Stmt::Expression(e) => {
                self.evaluate(e)?;
                Ok(())
            }
            Stmt::Var(name, init) => {
                self.environment
                    .define(name.value().to_owned(), self.evaluate(init)?);
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::*;
    use crate::parse::*;

    #[test]
    fn fooasd() {
        let mut p = Parser::new(Lexer::new(
            // "print 3 + 4; print 2 / 3; print true; print \"foo\" + \"bar\";",
            "var a = 5; print a",
        ));
        let mut i = Interpreter::new();
        // println!("{}", Interpreter::evaluate(p.next)
        i.execute(p.next().unwrap());
        // i.execute(p.next().unwrap());
        // i.execute(p.next().unwrap());
        // i.execute(p.next().unwrap());
    }
}
