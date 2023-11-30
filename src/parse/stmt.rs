use super::expr::Expr;

enum Stmt {
    Expression(Expr),
    Print(Expr),
}
