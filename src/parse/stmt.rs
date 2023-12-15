use super::expr::Expr;
use crate::lex::Token;
use crate::Located;

#[derive(Debug)]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Var(Located<String>, Expr),
}
