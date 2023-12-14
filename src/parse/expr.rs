use crate::lex::Token;
use crate::Located;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum Literal {
    Boolean(bool),
    Number(f64),
    String(String),
    Nil,
}

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Boolean(v) => write!(f, "{}", v),
            Self::Number(v) => write!(f, "{}", v),
            Self::String(v) => write!(f, "{}", v),
            Self::Nil => write!(f, "null"),
        }
    }
}

#[derive(Debug)]
pub enum Expr {
    Binary(Box<Expr>, Located<Token>, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Literal),
    Unary(Located<Token>, Box<Expr>),
}
