#![feature(test)]
mod interpret;
mod lex;
mod parse;
use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};

#[derive(Clone)]
enum Position {
    Coords(usize, usize),
    Eof,
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Coords(row, col) => write!(f, "{}:{}", row, col),
            Self::Eof => write!(f, "eof"),
        }
    }
}

#[derive(Clone)]
pub struct Located<T> {
    pos: Position,
    value: T,
}

impl<T> Located<T> {
    pub fn at_coords(row: usize, col: usize, value: T) -> Self {
        Self {
            pos: Position::Coords(row, col),
            value,
        }
    }

    pub fn at_eof(value: T) -> Self {
        Self {
            pos: Position::Eof,
            value,
        }
    }

    pub fn co_locate<L>(&self, value: L) -> Located<L> {
        Located {
            pos: self.pos.clone(),
            value,
        }
    }

    pub fn value(&self) -> &T {
        &self.value
    }
}

impl<D: Debug> Debug for Located<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {:?}", self.pos, self.value)
    }
}

impl<E: Error> Display for Located<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.pos, self.value)
    }
}

impl<E: Error> Error for Located<E> {}

mod error {
    use std::error::Error;
    use std::fmt::{Display, Formatter, Result};

    #[derive(Debug, Clone)]
    pub enum Syntax {
        Unclosed,
        StrayCharacter(char),
        ExpectedExpression,
        UnclosedGrouping,
        UnClosedExprStatement,
        ExpectedVariableName,
        ExpectedFunctionName,
        InvalidAssignmentTarget,
        UnclosedBlock,
        UnopenedBlock,
        ExpectedControlLeftParen,
        ExpectedControlRightParen,
        ExpectedSemicolonAfterForCondition,
        UnclosedArgumentsList,
        TooManyArguments,
        ExpectedFunctionLeftParen,
        ExpectedParameterName,
    }

    impl Display for Syntax {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            match self {
                Self::Unclosed => write!(f, "expected '\"' at the end of string"),
                Self::StrayCharacter(c) => write!(f, "stray {} in program", c),
                Self::ExpectedExpression => write!(f, "expected expression"),
                Self::UnclosedGrouping => {
                    write!(f, "expected ')' at the end of grouping expression")
                }
                Self::UnClosedExprStatement => {
                    write!(f, "expected ';' at the end of statement")
                }
                Self::ExpectedVariableName => write!(f, "expected variable name"),
                Self::InvalidAssignmentTarget => write!(f, "invalid assignment target"),
                Self::UnclosedBlock => write!(f, "expected '}}' at the end of block"),
                Self::ExpectedControlLeftParen => {
                    write!(f, "expected '(' after control statement")
                }
                Self::ExpectedControlRightParen => {
                    write!(f, "expected ')' before control statement body")
                }
                Self::ExpectedSemicolonAfterForCondition => {
                    write!(f, "expected ';' after for loop condition")
                }
                Self::UnclosedArgumentsList => {
                    write!(f, "expected ')' after argument list")
                }
                Self::TooManyArguments => {
                    write!(f, "can't have more than 255 arguments")
                }
                Self::ExpectedFunctionName => {
                    write!(f, "expected function name")
                }
                Self::ExpectedFunctionLeftParen => {
                    write!(f, "expected '(' after function name")
                }
                Self::ExpectedParameterName => {
                    write!(f, "expected parameter name")
                }
                Self::UnopenedBlock => {
                    write!(f, "expected '{{' before block")
                }
            }
        }
    }

    #[derive(Debug, Clone)]
    pub enum Runtime {
        ExpectedNumber,
        ExpectedNumbers,
        ExpectedNumbersOrStrings,
        UndefinedVariable(String),
        NotCallable,
        WrongArity(u8, u8),
    }

    impl Display for Runtime {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            match self {
                Self::ExpectedNumber => write!(f, "operand must be number"),
                Self::ExpectedNumbers => write!(f, "operands must be numbers"),
                Self::ExpectedNumbersOrStrings => {
                    write!(f, "operands must be either all numbers or all strings")
                }
                Self::UndefinedVariable(s) => {
                    write!(f, "variable '{}' is not defined", s)
                }
                Self::NotCallable => {
                    write!(f, "can only call functions and classes")
                }
                Self::WrongArity(expected, actual) => {
                    write!(f, "expected {} arguments, got {}", expected, actual)
                }
            }
        }
    }

    impl Error for Runtime {}
}

use interpret::{Environment, Interpreter};

use crate::parse::Function;
use std::rc::Rc;

#[derive(Clone)]
enum LoxFunction {
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
        environment: &mut Environment,
        arguments: Vec<Object>,
    ) -> Result<Object, Located<error::Runtime>> {
        environment.nest();
        match self {
            Self::User { declaration } => {
                environment.nest();
                for (value, name) in arguments.into_iter().zip(declaration.params.iter()) {
                    environment.define(name, value);
                }
                Interpreter::execute_block(&declaration.body, environment)?;
                environment.unnest().unwrap();
                // ?
                Ok(Object::Nil)
            }
            Self::Foreign { arity, f } => Ok(f(arguments)),
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

#[derive(Clone)]
enum Object {
    Boolean(bool),
    Number(f64),
    String(String),
    Nil,
    Function(Rc<LoxFunction>),
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
