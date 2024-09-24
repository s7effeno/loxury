pub mod chunk;
pub mod compiler;
mod lex;
mod location;
pub mod vm;

use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};

#[derive(Debug, Clone)]
pub enum CompileError {
    UnclosedString,
    StrayChar(char),
    UnclosedGrouping,
    ExpectedExpression,
    UnclosedStatement,
    ExpectedVariableName,
    InvalidAssignmentTarget,
    UnclosedBlock,
    TooManyLocals,
    VariableRedeclaration(String),
    SelfReferencialVariableInitializer(String),
}

impl Display for CompileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnclosedString => write!(f, "expected '\"' at the end of string"),
            Self::StrayChar(c) => write!(f, "stray '{}' in program", c),
            Self::UnclosedGrouping => write!(f, "expected ')' after expression"),
            Self::ExpectedExpression => write!(f, "expected expression"),
            Self::UnclosedStatement => write!(f, "expected ';' at the end of statement"),
            Self::ExpectedVariableName => write!(f, "expected variable name"),
            Self::InvalidAssignmentTarget => write!(f, "invalid assignment target"),
            Self::UnclosedBlock => write!(f, "expected '}}' at the end of block"),
            Self::TooManyLocals => write!(f, "can't have more than 256 local variables"),
            Self::VariableRedeclaration(v) => {
                write!(f, "variable '{}' already declared in this scope", v)
            }
            Self::SelfReferencialVariableInitializer(v) => {
                write!(f, "can't read local variable '{}' in its own initalizer", v)
            }
        }
    }
}

impl Error for CompileError {}

#[derive(Debug, Clone)]
pub enum RunError {
    ExpectedNumber,
    ExpectedNumbers,
    ExpectedNumbersOrStrings,
    UndefinedVariable(String),
}

impl Display for RunError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExpectedNumber => write!(f, "operand must be a number"),
            Self::ExpectedNumbers => write!(f, "operands must be numbers"),
            Self::ExpectedNumbersOrStrings => write!(f, "operands must be numbers or strings"),
            Self::UndefinedVariable(v) => write!(f, "variable {} is not defined", v),
        }
    }
}

impl Error for RunError {}
