use crate::error::Syntax as SyntaxError;
use crate::interpret::Interpreter;
use crate::parse::{Expr, Function, Literal, Stmt};
use crate::Located;
use std::collections::HashMap;
use std::mem;

enum FunctionKind {
    None,
    Function,
    Initializer,
    Method,
}

enum ClassKind {
    None,
    Class,
    SubClass,
}

pub struct Resolver<'a> {
    scopes: Vec<HashMap<&'a str, bool>>,
    interpreter: &'a mut Interpreter,
    errors: Vec<Located<SyntaxError>>,
    current_function: FunctionKind,
    current_class: ClassKind,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            scopes: Vec::new(),
            interpreter,
            errors: Vec::new(),
            current_function: FunctionKind::None,
            current_class: ClassKind::None,
        }
    }

    pub fn resolve(&mut self, statements: &'a [Stmt]) -> Result<(), ()> {
        self._resolve(statements);
        if self.errors.len() > 0 {
            Err(())
        } else {
            Ok(())
        }
    }

    fn _resolve(&mut self, statements: &'a [Stmt]) {
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
                self._resolve(b);
                self.end_scope();
            }
            Stmt::Expression(e) => self.resolve_expr(e),
            Stmt::Function(f) => {
                self.declare(&f.name);
                self.define(f.name.value());
                self.resolve_function(f, FunctionKind::Function);
            }
            Stmt::Print(e) => self.resolve_expr(e),
            Stmt::Return(loc, e) => {
                match self.current_function {
                    FunctionKind::None => self.error(loc.co_locate(SyntaxError::TopLevelReturn)),
                    FunctionKind::Initializer if !matches!(e, Expr::Literal(Literal::Nil)) => {
                        self.error(loc.co_locate(SyntaxError::InitializerReturn))
                    }
                    _ => (),
                }
                self.resolve_expr(e);
            }
            Stmt::Var(name, init) => {
                self.declare(name);
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
            Stmt::Class(name, superclass, methods) => {
                let enclosing_class = mem::replace(&mut self.current_class, ClassKind::Class);
                self.declare(&name);
                self.define(name.value());
                if let Some(superclass) = superclass {
                    self.current_class = ClassKind::SubClass;
                    let Expr::Variable(superclass_name) = superclass else {
                        unreachable!()
                    };
                    if superclass_name.value() == name.value() {
                        self.error(name.co_locate(SyntaxError::SelfInheritingClass));
                    }
                    self.resolve_expr(superclass);

                    self.begin_scope();
                    self.scopes.last_mut().unwrap().insert("super", true);
                }
                self.begin_scope();
                self.scopes.last_mut().unwrap().insert("this", true);
                for method in methods {
                    let declaration = if method.name.value() == "init" {
                        FunctionKind::Initializer
                    } else {
                        FunctionKind::Method
                    };

                    self.resolve_function(method, declaration);
                }
                self.end_scope();
                if superclass.is_some() {
                    self.end_scope();
                }
                self.current_class = enclosing_class;
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
                    self.error(
                        name.co_locate(SyntaxError::SelfReferencialVariableInitializer(
                            name.value().into(),
                        )),
                    );
                } else {
                    self.resolve_local(expr, name.value());
                }
            }
            Expr::Get(e, _) => {
                self.resolve_expr(e);
            }
            Expr::Set(object, _, value) => {
                self.resolve_expr(object);
                self.resolve_expr(value);
            }
            Expr::This(loc) => {
                if let ClassKind::None = self.current_class {
                    self.error(loc.co_locate(SyntaxError::ThisOutsideClass));
                }
                self.resolve_local(expr, "this");
            }
            Expr::Super(loc, _) => {
                match self.current_class {
                    ClassKind::None => self.error(loc.co_locate(SyntaxError::SuperOutsideClass)),
                    ClassKind::Class => self.error(loc.co_locate(SyntaxError::NoSuperClass)),
                    ClassKind::SubClass => (),
                }
                self.resolve_local(expr, "super");
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

    fn resolve_function(&mut self, function: &'a Function, kind: FunctionKind) {
        let enclosing_function = mem::replace(&mut self.current_function, kind);
        self.begin_scope();
        for param in function.params.iter() {
            self.declare(param);
            self.define(param.value());
        }
        self._resolve(&function.body);
        self.end_scope();
        self.current_function = enclosing_function;
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new())
    }

    fn end_scope(&mut self) {
        self.scopes.pop().unwrap();
    }

    fn declare(&mut self, name: &'a Located<String>) {
        if let Err(e) = match self.scopes.last_mut() {
            Some(s) if s.contains_key(name.value() as &str) => {
                Err(name.co_locate(SyntaxError::VariableRedeclaration(name.value().to_string())))
            }
            Some(s) => {
                s.insert(name.value(), false);
                Ok(())
            }
            _ => Ok(()),
        } {
            self.error(e);
        }
    }

    fn define(&mut self, name: &'a str) {
        self.scopes.last_mut().map(|s| s.insert(name, true));
    }

    fn error(&mut self, error: Located<SyntaxError>) {
        self.errors.push(error);
    }
}
