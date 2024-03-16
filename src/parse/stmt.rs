use super::expr::Expr;
use crate::Located;

#[derive(Debug)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expression(Expr),
    Function(Function),
    Print(Expr),
    Var(Located<String>, Option<Expr>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
}

#[derive(Debug)]
pub struct Function {
    name: String,
    params: Vec<String>,
    body: Vec<Stmt>,
}

impl Function {
    pub fn arity(&self) -> usize {
        self.params.len()
    }
}
