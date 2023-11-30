use crate::lex::Token;

pub enum Literal {
    Boolean(bool),
    Number(f64),
    String(String),
}

pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Option<Literal>),
    Unary(Token, Box<Expr>),
}
