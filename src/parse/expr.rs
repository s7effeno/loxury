use crate::lex::Token;

#[derive(Debug)]
pub enum Literal {
    Boolean(bool),
    Number(f64),
    String(String),
}

#[derive(Debug)]
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Option<Literal>),
    Unary(Token, Box<Expr>),
}
