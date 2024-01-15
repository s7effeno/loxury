use super::expr::Expr;
use crate::Located;

#[derive(Debug)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expression(Expr),
    Print(Expr),
    Var(Located<String>, Option<Expr>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
}
