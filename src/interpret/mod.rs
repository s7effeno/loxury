use crate::lex::Token;
use crate::parse::{Expr, Literal, Stmt};

pub struct Interpreter;

impl Interpreter {
    fn evaluate(expr: Expr) -> Option<Literal> {
        match expr {
            Expr::Literal(e) => e,
            Expr::Grouping(e) => Self::evaluate(*e),
            Expr::Unary(op, e) => {
                let right = Self::evaluate(*e);
                match op {
                    Token::Minus => {
                        if let Some(Literal::Number(n)) = right {
                            Some(Literal::Number(-n))
                        } else {
                            panic!();
                        }
                    }
                    Token::Bang => {
                        Some(Literal::Boolean({
                            match right {
                                None => false,
                                Some(Literal::Boolean(b)) => b,
                                _ => true
                            }
                        }))
                    }
                    _ => unreachable!(),
                }
            }
            Expr::Binary(l, op, r) => {
                let left = Self::evaluate(*l);
                let right = Self::evaluate(*r);
                match op {
                    Token::Minus => {
                        let Some(Literal::Number(left)) = left else { panic!() };
                        let Some(Literal::Number(right)) = right else { panic!() };
                        Some(Literal::Number(left - right))
                    }
                    Token::Slash => {
                        let Some(Literal::Number(left)) = left else { panic!() };
                        let Some(Literal::Number(right)) = right else { panic!() };
                        Some(Literal::Number(left / right))
                    }
                    Token::Star => {
                        let Some(Literal::Number(left)) = left else { panic!() };
                        let Some(Literal::Number(right)) = right else { panic!() };
                        Some(Literal::Number(left * right))
                    }
                    _ => unreachable!()
                }
            }
        }
    }
}
