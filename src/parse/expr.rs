use crate::lex::Token;
use crate::Located;
use std::fmt::{Debug, Display, Formatter, Result};

#[derive(Debug, Clone)]
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
            Self::Nil => write!(f, "nil"),
            // Self::Function(name, _, _) => write!(f, "<fn {}>", name),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Expr {
    Assign(Located<String>, Box<Expr>),
    Binary(Box<Expr>, Located<Token>, Box<Expr>),
    Call(Box<Expr>, Located<Token>, Vec<Expr>),
    Get(Box<Expr>, Located<String>),
    Grouping(Box<Expr>),
    Literal(Literal),
    Logical(Box<Expr>, Located<Token>, Box<Expr>),
    Unary(Located<Token>, Box<Expr>),
    Variable(Located<String>),
    Set(Box<Expr>, Located<String>, Box<Expr>),
    // `Located<()>` only stores `this` location
    This(Located<()>),
}
