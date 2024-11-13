pub mod chunk;
pub mod compiler;
mod gc;
mod lex;
mod location;
pub mod vm;

use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};
use std::mem::{self, MaybeUninit};
use std::{ptr, slice};

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
    UnopenedBlock,
    TooManyLocals,
    VariableRedeclaration(String),
    SelfReferencialVariableInitializer(String),
    ExpectedControlLeftParen,
    ExpectedControlRightParen,
    JumpTooWide,
    ExpectedForClauseSeparator,
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
            Self::UnopenedBlock => write!(f, "expected '{{' before a block"),
            Self::TooManyLocals => write!(f, "can't have more than 256 local variables"),
            Self::VariableRedeclaration(v) => {
                write!(f, "variable '{}' already declared in this scope", v)
            }
            Self::SelfReferencialVariableInitializer(v) => {
                write!(f, "can't read local variable '{}' in its own initalizer", v)
            }
            Self::ExpectedControlLeftParen => {
                write!(f, "expected '(' after control keyword")
            }
            Self::ExpectedControlRightParen => write!(f, "expected ') after control clause"),
            Self::JumpTooWide => write!(f, "too much code to jump over"),
            Self::ExpectedForClauseSeparator => write!(f, "expected ';' to separate for clauses"),
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

struct ArrayVec<T, const N: usize> {
    values: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> ArrayVec<T, N> {
    fn new() -> Self {
        Self {
            values: [ const { MaybeUninit::uninit() }; N],
            len: 0,
        }
    }

    fn push(&mut self, value: T) {
        (self.values[self.len as usize]).write(value);
        self.len += 1;
    }

    fn pop(&mut self) -> Option<T> {
        self.len = self.len.checked_sub(1)?;
        Some(unsafe { self.values[self.len].assume_init_read() })
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.values.as_ptr() as *const T, self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.values.as_mut_ptr() as *mut T, self.len) }
    }
}

impl<T, const N: usize> Drop for ArrayVec<T, N> {
    fn drop(&mut self) {
        let len = self.len;
        self.len = 0;
        for i in 0..len {
            unsafe { self.values[i].assume_init_drop() };
        }
    }
}

impl<T, const N: usize> std::ops::Deref for ArrayVec<T, N> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, const N: usize> std::ops::DerefMut for ArrayVec<T, N> {
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}
