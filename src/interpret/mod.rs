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

    fn is_truthy(val: Literal) -> bool {
        match val {
            Literal::Nil => false,
            Literal::Boolean(b) => b,
            _ => true,
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Literal, Located<RuntimeError>> {
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
            Expr::Literal(e) => Ok(e.clone()),
            Expr::Grouping(e) => self.evaluate(e),
            Expr::Unary(op, e) => {
                let right = self.evaluate(e)?;
                match op.value() {
                    Token::Minus => {
                        if let Literal::Number(n) = right {
                            Ok(Literal::Number(-n))
                        } else {
                            panic!();
                        }
                    }
                    Token::Bang => Ok(Literal::Boolean({ !Self::is_truthy(right) })),
                    _ => unreachable!(),
                }
            }
            Expr::Binary(l, op, r) => {
                let left = self.evaluate(l)?;
                let right = self.evaluate(r)?;
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
            Expr::Variable(name) => self
                .environment
                .get(name.value())
                .map(|l| l.clone())
                .map_err(|_| {
                    name.co_locate(RuntimeError::UndefinedVariable(name.value().to_owned()))
                }),
            Expr::Assign(name, value) => {
                let value = self.evaluate(value)?;
                self.environment
                    .assign(name.value(), value.clone())
                    .map(|_| value)
                    .map_err(|_| {
                        name.co_locate(RuntimeError::UndefinedVariable(name.value().to_owned()))
                    })
            }
            Expr::Logical(l, op, r) => {
                let left = self.evaluate(l)?;
                if let Token::Or = op.value() {
                    if Self::is_truthy(left.clone()) {
                        return Ok(left);
                    }
                } else {
                    if !Self::is_truthy(left.clone()) {
                        return Ok(left);
                    }
                }

                self.evaluate(r)
            }
        }
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), Located<RuntimeError>> {
        match stmt {
            Stmt::Print(e) => {
                println!("{}", self.evaluate(&e)?);
                Ok(())
            }
            Stmt::Expression(e) => {
                self.evaluate(&e)?;
                Ok(())
            }
            Stmt::Var(name, init) => {
                let init = self.evaluate(init.as_ref().unwrap_or(&Expr::Literal(Literal::Nil)))?;
                self.environment.define(name.value(), init);
                Ok(())
            }
            Stmt::Block(b) => {
                self.environment.nest();
                for s in b {
                    self.execute(s)?;
                }
                self.environment
                    .unnest()
                    // unreachable, the parser would spot it
                    .expect("no enclosing scope to revert to");
                Ok(())
            }
            Stmt::If(cond, branch_then, branch_else) => {
                if Self::is_truthy(self.evaluate(&cond)?) {
                    self.execute(branch_then)
                } else {
                    if let Some(branch_else) = branch_else {
                        self.execute(branch_else)?;
                        Ok(())
                    } else {
                        Ok(())
                    }
                }
            }
            Stmt::While(cond, body) => {
                while Self::is_truthy(self.evaluate(&cond)?) {
                    self.execute(body)?;
                }
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
            "
            var a = 0;
            var temp;

            for (var b = 1; a < 10000; b = temp + b) {
              print a;
              temp = a;
              a = b;
            }",
        ));
        let mut i = Interpreter::new();
        // println!("{}", Interpreter::evaluate(p.next)
        println!("{:?}", i.execute(&p.next().unwrap()));
        println!("{:?}", i.execute(&p.next().unwrap()));
        println!("{:?}", i.execute(&p.next().unwrap()));
    }
}
