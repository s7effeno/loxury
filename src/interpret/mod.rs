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
        Self::_evaluate(expr, &mut self.environment)
    }

    fn _evaluate(
        expr: &Expr,
        environment: &mut Environment,
    ) -> Result<Literal, Located<RuntimeError>> {
        fn is_equal(left: Literal, right: Literal) -> bool {
            match (left, right) {
                (Literal::Nil, Literal::Nil) => true,
                (Literal::Boolean(left), Literal::Boolean(right)) => left == right,
                (Literal::Number(left), Literal::Number(right)) => left == right,
                (Literal::String(left), Literal::String(right)) => left == right,
                // TODO: add remaining checks
                _ => false,
            }
        }

        match expr {
            Expr::Literal(e) => Ok(e.clone()),
            Expr::Grouping(e) => Self::_evaluate(e, environment),
            Expr::Unary(op, e) => {
                let right = Self::_evaluate(e, environment)?;
                match op.value() {
                    Token::Minus => {
                        if let Literal::Number(n) = right {
                            Ok(Literal::Number(-n))
                        } else {
                            panic!();
                        }
                    }
                    Token::Bang => Ok(Literal::Boolean(!Self::is_truthy(right))),
                    _ => unreachable!(),
                }
            }
            Expr::Binary(l, op, r) => {
                let left = Self::_evaluate(l, environment)?;
                let right = Self::_evaluate(r, environment)?;
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
            Expr::Variable(name) => {
                environment
                    .get(name.value())
                    .map(|l| l.clone())
                    .map_err(|_| {
                        name.co_locate(RuntimeError::UndefinedVariable(name.value().to_owned()))
                    })
            }
            Expr::Assign(name, value) => {
                let value = Self::_evaluate(value, environment)?;
                environment
                    .assign(name.value(), value.clone())
                    .map(|_| value)
                    .map_err(|_| {
                        name.co_locate(RuntimeError::UndefinedVariable(name.value().to_owned()))
                    })
            }
            Expr::Logical(l, op, r) => {
                let left = Self::_evaluate(l, environment)?;
                if let Token::Or = op.value() {
                    if Self::is_truthy(left.clone()) {
                        return Ok(left);
                    }
                } else {
                    if !Self::is_truthy(left.clone()) {
                        return Ok(left);
                    }
                }

                Self::_evaluate(r, environment)
            }
            Expr::Call(_, _, _) => todo!(),
        }
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), Located<RuntimeError>> {
        Self::_execute(stmt, &mut self.environment)
    }

    fn _execute(stmt: &Stmt, environment: &mut Environment) -> Result<(), Located<RuntimeError>> {
        match stmt {
            Stmt::Print(e) => {
                println!("{}", Self::_evaluate(&e, environment)?);
                Ok(())
            }
            Stmt::Expression(e) => {
                Self::_evaluate(&e, environment)?;
                Ok(())
            }
            Stmt::Var(name, init) => {
                let init = Self::_evaluate(
                    init.as_ref().unwrap_or(&Expr::Literal(Literal::Nil)),
                    environment,
                )?;
                environment.define(name.value(), init);
                Ok(())
            }
            Stmt::Block(b) => {
                Self::execute_block(b, environment);
                Ok(())
            }
            Stmt::If(cond, branch_then, branch_else) => {
                if Self::is_truthy(Self::_evaluate(&cond, environment)?) {
                    Self::_execute(branch_then, environment)
                } else {
                    if let Some(branch_else) = branch_else {
                        Self::_execute(branch_else, environment)?;
                        Ok(())
                    } else {
                        Ok(())
                    }
                }
            }
            Stmt::While(cond, body) => {
                while Self::is_truthy(Self::_evaluate(&cond, environment)?) {
                    Self::_execute(body, environment)?;
                }
                Ok(())
            }
            Stmt::Function(_) => todo!()
        }
    }

    fn execute_block(statements: &Vec<Stmt>, environment: &mut Environment) {
        environment.nest();
        for statement in statements {}
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
            var x = 6;
            var x;
            print x;
            for (var i = 0; i < 10; i = i + 1) {
                print i;
                for (var j = 0; j < 10; j = j + 1) {
                    print j;
                }
            }",
        ));
        let mut i = Interpreter::new();
        // println!("{}", Interpreter::evaluate(p.next)
        println!("{:?}", i.execute(&p.next().unwrap()));
        println!("{:?}", i.execute(&p.next().unwrap()));
        println!("{:?}", i.execute(&p.next().unwrap()));
        println!("{:?}", i.execute(&p.next().unwrap()));
    }
}
