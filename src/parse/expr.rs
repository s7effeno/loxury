use crate::lex::Token;

enum Literal {
    String(String),
    Number(f64),
}

enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Literal),
    Unary(Token, Box<Expr>),
}
