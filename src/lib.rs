#![feature(test)]
mod lex;
pub use lex::Lexer;
mod parse;
pub use parse::Parser;
mod resolver;
pub use resolver::Resolver;
mod interpret;
pub use interpret::Interpreter;
use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};

enum Either<A, B> {
    A(A),
    B(B),
}

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
        UnclosedStatement,
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
        SelfReferencialVariableInitializer,
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
                Self::UnclosedStatement => {
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
                Self::SelfReferencialVariableInitializer => {
                    write!(f, "can't read local variable in its own initalizer")
                }
            }
        }
    }

    impl Error for Syntax {}

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
