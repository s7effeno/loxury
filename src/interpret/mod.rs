use crate::lex::Token;
use crate::parse::{Expr, Literal, Stmt};

pub struct Interpreter;

impl Interpreter {
    fn evaluate(expr: Expr) -> Option<Literal> {
        fn is_equal(left: Option<Literal>, right: Option<Literal>) -> bool {
            match (left, right) {
                (None, None) => true,
                (None, _) => false,
                (_, None) => false,
                (Some(Literal::Boolean(left)), Some(Literal::Boolean(right))) => left == right,
                (Some(Literal::Number(left)), Some(Literal::Number(right))) => left == right,
                (Some(Literal::String(left)), Some(Literal::String(right))) => left == right,
                _ => false,
            }
        }

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
                    Token::Greater => {
                        let Some(Literal::Number(left)) = left else { panic!() };
                        let Some(Literal::Number(right)) = right else { panic!() };
                        Some(Literal::Boolean(left > right))
                    }
                    Token::GreaterEqual => {
                        let Some(Literal::Number(left)) = left else { panic!() };
                        let Some(Literal::Number(right)) = right else { panic!() };
                        Some(Literal::Boolean(left >= right))
                    }
                    Token::Less => {
                        let Some(Literal::Number(left)) = left else { panic!() };
                        let Some(Literal::Number(right)) = right else { panic!() };
                        Some(Literal::Boolean(left < right))
                    }
                    Token::LessEqual => {
                        let Some(Literal::Number(left)) = left else { panic!() };
                        let Some(Literal::Number(right)) = right else { panic!() };
                        Some(Literal::Boolean(left <= right))
                    }
                    Token::Minus => {
                        let Some(Literal::Number(left)) = left else { panic!() };
                        let Some(Literal::Number(right)) = right else { panic!() };
                        Some(Literal::Number(left - right))
                    }
                    Token::BangEqual => {
                        Some(Literal::Boolean(!is_equal(left, right)))
                    }
                    Token::EqualEqual => {
                        Some(Literal::Boolean(is_equal(left, right)))
                    }
                    Token::Plus => {
                        match (left, right) {
                            (Some(Literal::Number(left)), Some(Literal::Number(right))) => Some(Literal::Number(left + right)),
                            (Some(Literal::String(left)), Some(Literal::String(right))) => Some(Literal::String(left + &right)),
                            _ => panic!(),
                        }
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
