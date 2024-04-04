use super::expr::Expr;
use crate::Located;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expression(Expr),
    Function(Rc<Function>),
    Class(Located<String>, Vec<Rc<Function>>),
    Print(Expr),
    // .0 stores location of "return" lexeme
    // TODO: implement less hacky solution
    Return(Located<()>, Expr),
    Var(Located<String>, Option<Expr>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
}

#[derive(Clone, Debug)]
pub struct Function {
    pub name: Located<String>,
    pub params: Vec<Located<String>>,
    pub body: Vec<Stmt>,
}
