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
    values: [MaybeUninit<T>; u8::MAX as usize + 1],
    len: usize,
}

impl<T, const N: usize> ArrayVec<T, N> {
    fn new() -> Self {
        Self {
            values: unsafe { MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }

    fn push(&mut self, value: T) {
        assert_ne!(self.len, usize::MAX);
        (self.values[self.len as usize]).write(value);
        self.len += 1;
    }

    fn pop(&mut self) -> T {
        assert_ne!(self.len, 0);
        self.len -= 1;
        unsafe { ptr::read(mem::transmute(self.values.as_ptr().add(self.len))) }
    }

    fn last(&self) -> Option<&T> {
        let index = self.len - 1;
        self.get(index)
    }

    fn last_mut(&mut self) -> Option<&mut T> {
        let index = self.len - 1;
        self.get_mut(index)
    }

    fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            Some(unsafe { self.values[index as usize].assume_init_ref() })
        } else {
            None
        }
    }

    fn get_mut(&mut self, slot: usize) -> Option<&mut T> {
        assert!(slot < self.len);
        Some(unsafe { mem::transmute(&mut self.values[slot as usize]) })
    }

    fn iter(&self) -> std::slice::Iter<'_, T> {
        unsafe { slice::from_raw_parts(self.values.as_ptr() as *const T, self.len) }.iter()
    }
}
