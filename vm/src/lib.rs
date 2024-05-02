mod chunk;
mod compiler;
mod lex;
mod location;
mod vm;

use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};

#[derive(Debug, Clone)]
pub enum CompileError {
    UnclosedString,
    StrayChar(char),
    UnclosedGrouping,
    ExpectedExpression,
}

impl Display for CompileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnclosedString => write!(f, "expected '\"' at the end of string"),
            Self::StrayChar(c) => write!(f, "stray '{}' in program", c),
            Self::UnclosedGrouping => write!(f, "expected ')' after expression"),
            Self::ExpectedExpression => write!(f, "expected expression"),
        }
    }
}

impl Error for CompileError {}

#[derive(Debug, Clone)]
pub enum RunError {
    ExpectedNumbers,
}

impl Display for RunError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExpectedNumbers => write!(f, "operands must be numbers"),
        }
    }
}

impl Error for RunError {}
