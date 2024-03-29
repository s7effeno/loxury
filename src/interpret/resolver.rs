use super::Interpreter;
use crate::Located;
use crate::error::Syntax as SyntaxError;
use crate::parse::{Expr, Stmt};
use std::collections::HashMap;

pub struct Resolver<'a> {
    scopes: Vec<HashMap<&'a str, bool>>,
    interpreter: &'a mut Interpreter,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            scopes: Vec::new(),
            interpreter,
        }
    }

    pub fn resolve(&mut self, statements: &'a [Stmt]) {
        for stmt in statements {
            self.resolve_stmt(stmt);
        }
    }

    fn resolve_stmt(&mut self, stmt: &'a Stmt) {
        match stmt {
            Stmt::Block(_) => todo!(),
            Stmt::Expression(_) => todo!(),
            Stmt::Function(_) => todo!(),
            Stmt::Print(_) => todo!(),
            Stmt::Return(_) => todo!(),
            Stmt::Var(name, init) => {
                self.declare(name.value());
                if let Some(init) = init {
                    self.resolve_expr(init);
                }
                self.define(name.value());
            }
            Stmt::If(_, _, _) => todo!(),
            Stmt::While(_, _) => todo!(),
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) -> Result<(), Located<SyntaxError>> {
        match expr {
            Expr::Assign(_, _) => todo!(),
            Expr::Binary(_, _, _) => todo!(),
            Expr::Call(_, _, _) => todo!(),
            Expr::Grouping(_) => todo!(),
            Expr::Literal(_) => todo!(),
            Expr::Logical(_, _, _) => todo!(),
            Expr::Unary(_, _) => todo!(),
            Expr::Variable(name) => {
                if self
                    .scopes
                    .last_mut()
                    .filter(|s| s.get(name.value() as &str).is_some_and(|b| !b))
                    .is_some()
                {
                    Err(name.co_locate(SyntaxError::SelfReferencialVariableInitializer))
                } else {
                    self.resolve_local(expr, name.value());
                    Ok(())
                }
            }
        }
    }

    fn resolve_local(&mut self, expr: &Expr, name: &str) {
        let scopes_len = self.scopes.len();
        for (i, scope) in self.scopes.iter_mut().enumerate() {
            if scope.contains_key(name) {
                let depth = scopes_len - i - 1;
                self.interpreter.resolve(expr, depth);
            }
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new())
    }

    fn end_scope(&mut self) {
        self.scopes.pop().unwrap();
    }

    fn declare(&mut self, name: &'a str) {
        self.scopes.last_mut().map(|s| s.insert(name, false));
    }

    fn define(&mut self, name: &'a str) {
        self.scopes.last_mut().map(|s| s.insert(name, true));
    }
}
