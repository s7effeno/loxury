use super::expr::Expr;

pub enum Stmt {
    Expression(Expr),
    Print(Expr),
}
