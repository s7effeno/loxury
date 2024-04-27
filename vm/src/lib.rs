mod chunk;
mod compiler;
mod lex;
mod vm;

use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};

#[derive(Clone, Debug)]
pub enum Position {
    Coords(u16, u16),
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

#[derive(Debug)]
pub struct Located<T> {
    pos: Position,
    value: T,
}

impl<T> Located<T> {
    pub fn at_coords(row: u16, col: u16, value: T) -> Self {
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

impl<T: Clone> Clone for Located<T> {
    fn clone(&self) -> Self {
        Self {
            pos: self.pos.clone(),
            value: self.value.clone(),
        }
    }
}

impl<E: Error> Display for Located<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.pos, self.value)
    }
}

impl<E: Error> Error for Located<E> {}

#[derive(Debug, Clone)]
pub enum CompileError {
    UnclosedString,
    StrayChar(char),
    UnclosedGrouping,
}

impl Display for CompileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnclosedString => write!(f, "expected '\"' at the end of string"),
            Self::StrayChar(c) => write!(f, "stray '{}' in program", c),
            Self::UnclosedGrouping => write!(f, "expected ')' after expression"),
        }
    }
}

impl Error for CompileError {}
