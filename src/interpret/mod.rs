use crate::error::Runtime as RuntimeError;
use crate::lex::Token;
use crate::parse::Function;
use crate::parse::{Expr, Literal, Stmt};
use crate::Either;
use crate::Located;
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;
use std::time::UNIX_EPOCH;
use std::collections::HashMap;
use std::ptr;

mod environment;
mod resolver;
pub use environment::Environment;

type Unwinder = Either<Located<RuntimeError>, Object>;

impl From<Located<RuntimeError>> for Unwinder {
    fn from(value: Located<RuntimeError>) -> Self {
        Either::A(value)
    }
}

#[derive(Clone)]
pub enum Object {
    Boolean(bool),
    Number(f64),
    String(String),
    Nil,
    Function(Rc<LoxFunction>),
}

impl From<Literal> for Object {
    fn from(value: Literal) -> Self {
        match value {
            Literal::Boolean(v) => Self::Boolean(v),
            Literal::Nil => Self::Nil,
            Literal::Number(v) => Self::Number(v),
            Literal::String(s) => Self::String(s),
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean(v) => write!(f, "{}", v),
            Self::Number(v) => write!(f, "{}", v),
            Self::String(v) => write!(f, "{}", v),
            Self::Nil => write!(f, "nil"),
            Self::Function(v) => write!(f, "{}", v),
        }
    }
}

#[derive(Clone)]
pub enum LoxFunction {
    User {
        declaration: Function,
    },
    Foreign {
        arity: u8,
        f: fn(Vec<Object>) -> Object,
    },
}

impl LoxFunction {
    fn arity(&self) -> u8 {
        match self {
            Self::User { declaration } => declaration.params.len() as u8,
            Self::Foreign { arity, .. } => *arity,
        }
    }

    fn call(
        &self,
        environment: &Environment,
        arguments: Vec<Object>,
    ) -> Result<Object, Located<RuntimeError>> {
        match self {
            Self::User { declaration } => {
                let environment = environment.nest();
                for (value, name) in arguments.into_iter().zip(declaration.params.iter()) {
                    environment.define(name, value);
                }
                let ret = match Interpreter::execute_block(&declaration.body, &environment) {
                    Ok(()) => Ok(Object::Nil),
                    Err(Unwinder::B(ret)) => Ok(ret),
                    Err(Unwinder::A(err)) => Err(err),
                };
                ret
            }
            Self::Foreign { f, .. } => Ok(f(arguments)),
        }
    }
}

impl Display for LoxFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::User { declaration } => write!(f, "<fn {}>", declaration.name),
            Self::Foreign { .. } => write!(f, "<foreign fn>"),
        }
    }
}

impl From<Object> for Unwinder {
    fn from(value: Object) -> Self {
        Either::B(value)
    }
}

pub struct Interpreter {
    globals: Environment,
    locals: HashMap<*const Expr, usize>,
    environment: Environment,
}

impl Interpreter {
    fn new() -> Self {
        let globals = Environment::new();
        globals.define(
            "clock",
            Object::Function(
                LoxFunction::Foreign {
                    arity: 0,
                    f: |_| Object::Number(UNIX_EPOCH.elapsed().unwrap().as_millis() as f64),
                }
                .into(),
            ),
        );

        let environment = globals.clone();
        Self {
            globals,
            locals: HashMap::new(),
            environment,
        }
    }

    fn is_truthy(val: &Object) -> bool {
        match val {
            Object::Nil => false,
            Object::Boolean(b) => *b,
            _ => true,
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Object, Located<RuntimeError>> {
        Self::_evaluate(expr, &self.environment)
    }

    fn _evaluate(expr: &Expr, environment: &Environment) -> Result<Object, Located<RuntimeError>> {
        // maybe implement directly in object?
        fn is_equal(left: Object, right: Object) -> bool {
            match (left, right) {
                (Object::Nil, Object::Nil) => true,
                (Object::Boolean(left), Object::Boolean(right)) => left == right,
                (Object::Number(left), Object::Number(right)) => left == right,
                (Object::String(left), Object::String(right)) => left == right,
                // TODO: add remaining checks
                _ => false,
            }
        }

        match expr {
            Expr::Literal(e) => Ok(e.clone().into()),
            Expr::Grouping(e) => Self::_evaluate(e, environment),
            Expr::Unary(op, e) => {
                let right = Self::_evaluate(e, environment)?;
                match op.value() {
                    Token::Minus => {
                        if let Object::Number(n) = right {
                            Ok(Object::Number(-n))
                        } else {
                            Err(op.co_locate(RuntimeError::ExpectedNumber))
                        }
                    }
                    Token::Bang => Ok(Object::Boolean(!Self::is_truthy(&right))),
                    _ => unreachable!(),
                }
            }
            Expr::Binary(l, op, r) => {
                let left = Self::_evaluate(l, environment)?;
                let right = Self::_evaluate(r, environment)?;
                match op.value() {
                    Token::Greater => {
                        if let (Object::Number(left), Object::Number(right)) = (left, right) {
                            Ok(Object::Boolean(left > right))
                        } else {
                            Err(op.co_locate(RuntimeError::ExpectedNumbers))
                        }
                    }
                    Token::GreaterEqual => {
                        if let (Object::Number(left), Object::Number(right)) = (left, right) {
                            Ok(Object::Boolean(left >= right))
                        } else {
                            Err(op.co_locate(RuntimeError::ExpectedNumbers))
                        }
                    }
                    Token::Less => {
                        if let (Object::Number(left), Object::Number(right)) = (left, right) {
                            Ok(Object::Boolean(left < right))
                        } else {
                            Err(op.co_locate(RuntimeError::ExpectedNumbers))
                        }
                    }
                    Token::LessEqual => {
                        if let (Object::Number(left), Object::Number(right)) = (left, right) {
                            Ok(Object::Boolean(left <= right))
                        } else {
                            Err(op.co_locate(RuntimeError::ExpectedNumbers))
                        }
                    }
                    Token::Minus => {
                        if let (Object::Number(left), Object::Number(right)) = (left, right) {
                            Ok(Object::Number(left - right))
                        } else {
                            Err(op.co_locate(RuntimeError::ExpectedNumbers))
                        }
                    }
                    Token::BangEqual => Ok(Object::Boolean(!is_equal(left, right))),
                    Token::EqualEqual => Ok(Object::Boolean(is_equal(left, right))),
                    Token::Plus => match (left, right) {
                        (Object::Number(left), Object::Number(right)) => {
                            Ok(Object::Number(left + right))
                        }
                        (Object::String(left), Object::String(right)) => {
                            Ok(Object::String(left + &right))
                        }
                        _ => Err(op.co_locate(RuntimeError::ExpectedNumbersOrStrings)),
                    },
                    Token::Slash => {
                        if let (Object::Number(left), Object::Number(right)) = (left, right) {
                            Ok(Object::Number(left / right))
                        } else {
                            Err(op.co_locate(RuntimeError::ExpectedNumbers))
                        }
                    }
                    Token::Star => {
                        if let (Object::Number(left), Object::Number(right)) = (left, right) {
                            Ok(Object::Number(left * right))
                        } else {
                            Err(op.co_locate(RuntimeError::ExpectedNumbers))
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Expr::Variable(name) => {
                environment
                    .get(name.value())
                    .map(|l| l.clone())
                    .map_err(|_| {
                        name.co_locate(RuntimeError::UndefinedVariable(name.value().to_owned()))
                    })
            }
            Expr::Assign(name, value) => {
                let value = Self::_evaluate(value, environment)?;
                environment
                    .assign(name.value(), value.clone())
                    .map(|_| value)
                    .map_err(|_| {
                        name.co_locate(RuntimeError::UndefinedVariable(name.value().to_owned()))
                    })
            }
            Expr::Logical(l, op, r) => {
                let left = Self::_evaluate(l, environment)?;
                if let Token::Or = op.value() {
                    if Self::is_truthy(&left) {
                        return Ok(left);
                    }
                } else {
                    if !Self::is_truthy(&left) {
                        return Ok(left);
                    }
                }

                Self::_evaluate(r, environment)
            }
            Expr::Call(callee, paren, args) => {
                let callee = Self::_evaluate(callee, environment)?;
                let mut expanded_args = Vec::with_capacity(args.len());
                for arg in args {
                    expanded_args.push(Self::_evaluate(arg, environment)?);
                }
                let Object::Function(f) = callee else {
                    return Err(paren.co_locate(RuntimeError::NotCallable));
                };
                let expected = f.arity();
                let actual = args.len() as u8;
                if expected != actual {
                    return Err(paren.co_locate(RuntimeError::WrongArity(expected, actual)));
                }
                f.call(environment, expanded_args)
            }
        }
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), Unwinder> {
        Self::_execute(stmt, &self.environment)
    }

    fn _execute(stmt: &Stmt, environment: &Environment) -> Result<(), Unwinder> {
        match stmt {
            Stmt::Print(e) => {
                println!("{}", Self::_evaluate(&e, environment)?);
                Ok(())
            }
            Stmt::Expression(e) => {
                Self::_evaluate(&e, environment)?;
                Ok(())
            }
            Stmt::Var(name, init) => {
                let init = Self::_evaluate(
                    init.as_ref().unwrap_or(&Expr::Literal(Literal::Nil)),
                    environment,
                )?;
                environment.define(name.value(), init);
                Ok(())
            }
            Stmt::Block(b) => {
                Self::execute_block(b, &environment.nest())?;
                Ok(())
            }
            Stmt::If(cond, branch_then, branch_else) => {
                if Self::is_truthy(&Self::_evaluate(&cond, environment)?) {
                    Self::_execute(branch_then, environment)
                } else {
                    if let Some(branch_else) = branch_else {
                        Self::_execute(branch_else, environment)?;
                        Ok(())
                    } else {
                        Ok(())
                    }
                }
            }
            Stmt::While(cond, body) => {
                while Self::is_truthy(&Self::_evaluate(&cond, environment)?) {
                    Self::_execute(body, environment)?;
                }
                Ok(())
            }
            Stmt::Function(f) => {
                environment.define(
                    &f.name,
                    Object::Function(
                        LoxFunction::User {
                            declaration: f.clone(),
                        }
                        .into(),
                    ),
                );
                Ok(())
            }
            Stmt::Return(v) => {
                let v = Self::_evaluate(&v, environment)?;
                Err(v.into())
            }
        }
    }

    fn resolve(&mut self, expr: &Expr, depth: usize) {
        self.locals.insert(expr as *const Expr, depth);
    }

    fn execute_block(statements: &Vec<Stmt>, environment: &Environment) -> Result<(), Unwinder> {
        for statement in statements {
            Self::_execute(statement, &environment)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex::*;
    use crate::parse::*;

    #[test]
    fn fooasd() {
        let mut p = Parser::new(Lexer::new(
            "
            var x = 6;
            var x;
            print x;
            for (var i = 0; i < 10; i = i + 1) {
                print i;
                for (var j = 0; j < 10; j = j + 1) {
                    print j;
                }
            }",
        ));
        let mut i = Interpreter::new();
        // println!("{}", Interpreter::evaluate(p.next)
        /*println!("{:?}", i.execute(&p.next().unwrap()));
        println!("{:?}", i.execute(&p.next().unwrap()));
        println!("{:?}", i.execute(&p.next().unwrap()));
        println!("{:?}", i.execute(&p.next().unwrap()));*/
        i.execute(&p.next().unwrap());
        i.execute(&p.next().unwrap());
        i.execute(&p.next().unwrap());
        i.execute(&p.next().unwrap());
    }

    #[test]
    fn functions() {
        let mut p = Parser::new(Lexer::new(
            "
            fun sayHi(first, last) {
              print \"Hi, \" + first + \" \" + last + \"!\";
            }

            sayHi(\"Dear\", \"Reader\");
            ",
        ));
        let mut i = Interpreter::new();
        /*println!("{:?}", i.execute(&p.next().unwrap()));
        println!("{:?}", i.execute(&p.next().unwrap()));*/
        i.execute(&p.next().unwrap());
        i.execute(&p.next().unwrap());
    }

    #[test]
    fn function_ret() {
        let mut p = Parser::new(Lexer::new(
            "
            fun sum(a, b) {
                while (true) {
                    while (true) {
                        if (true) {
                            return a + b;
                        }
                    }
                }
            }
            print sum(5, 6);
            ",
        ));
        let mut i = Interpreter::new();
        i.execute(&p.next().unwrap());
        i.execute(&p.next().unwrap());
    }

    #[test]
    fn fib() {
        let mut p = Parser::new(Lexer::new(
            "
            fun fib(n) {
              if (n <= 1) return n;
              return fib(n - 2) + fib(n - 1);
            }

            for (var i = 0; i < 30; i = i + 1) {
              print fib(i);
            }
            ",
        ));
        let mut i = Interpreter::new();
        i.execute(&p.next().unwrap());
        i.execute(&p.next().unwrap());
    }

    #[test]
    fn block() {
        let mut p = Parser::new(Lexer::new(
            "
            var a = 5;
            {
            var a = a + 1;
            print a;
            }
            print a;
            ",
        ));
        let mut i = Interpreter::new();
        i.execute(&p.next().unwrap());
        i.execute(&p.next().unwrap());
        i.execute(&p.next().unwrap());
    }
}
