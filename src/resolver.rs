use crate::error::Syntax as SyntaxError;
use crate::interpret::Interpreter;
use crate::parse::{Expr, Function, Stmt};
use crate::Located;
use std::collections::HashMap;

pub struct Resolver<'a> {
    scopes: Vec<HashMap<&'a str, bool>>,
    interpreter: &'a mut Interpreter,
    errors: Vec<Located<SyntaxError>>,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            scopes: Vec::new(),
            interpreter,
            errors: Vec::new(),
        }
    }

    pub fn resolve(&mut self, statements: &'a [Stmt]) {
        for stmt in statements {
            self.resolve_stmt(stmt);
        }
    }

    pub fn errors(&'a self) -> &'a [Located<SyntaxError>] {
        &self.errors
    }

    fn resolve_stmt(&mut self, stmt: &'a Stmt) {
        match stmt {
            Stmt::Block(b) => {
                self.begin_scope();
                self.resolve(b);
                self.end_scope();
            }
            Stmt::Expression(e) => self.resolve_expr(e),
            Stmt::Function(f) => {
                self.declare(&f.name);
                self.define(&f.name);
                self.resolve_function(f);
            }
            Stmt::Print(e) => self.resolve_expr(e),
            Stmt::Return(e) => self.resolve_expr(e),
            Stmt::Var(name, init) => {
                self.declare(name.value());
                if let Some(init) = init {
                    self.resolve_expr(init);
                }
                self.define(name.value());
            }
            Stmt::If(cond, branch_then, branch_else) => {
                self.resolve_expr(cond);
                self.resolve_stmt(branch_then);
                if let Some(branch_else) = branch_else {
                    self.resolve_stmt(branch_else);
                }
            }
            Stmt::While(cond, body) => {
                self.resolve_expr(cond);
                self.resolve_stmt(body);
            }
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Assign(name, value) => {
                self.resolve_expr(value);
                self.resolve_local(expr, name.value());
            }
            Expr::Binary(l, _, r) => {
                self.resolve_expr(l);
                self.resolve_expr(r)
            }
            Expr::Call(callee, _, args) => {
                self.resolve_expr(callee);
                for arg in args {
                    self.resolve_expr(arg);
                }
            }
            Expr::Grouping(e) => self.resolve_expr(e),
            Expr::Literal(_) => {}
            Expr::Logical(l, _, r) => {
                self.resolve_expr(l);
                self.resolve_expr(r);
            }
            Expr::Unary(_, e) => self.resolve_expr(e),
            Expr::Variable(name) => {
                if self
                    .scopes
                    .last_mut()
                    .filter(|s| s.get(name.value() as &str).is_some_and(|b| !b))
                    .is_some()
                {
                    self.error(name.co_locate(SyntaxError::SelfReferencialVariableInitializer));
                } else {
                    self.resolve_local(expr, name.value());
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
                return;
            }
        }
    }

    fn resolve_function(&mut self, function: &'a Function) {
        self.begin_scope();
        for param in function.params.iter() {
            self.declare(param);
            self.define(param);
        }
        self.resolve(&function.body);
        self.end_scope();
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new())
    }

    fn end_scope(&mut self) {
        self.scopes.pop().unwrap();
    }

    fn declare(&mut self, name: &'a str) {
        let scope = self.scopes.last_mut();
        self.scopes.last_mut().map(|s| s.insert(name, false));
    }

    fn define(&mut self, name: &'a str) {
        self.scopes.last_mut().map(|s| s.insert(name, true));
    }

    fn error(&mut self, error: Located<SyntaxError>) {
        self.errors.push(error);
    }
}
