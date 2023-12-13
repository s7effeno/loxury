#![feature(test)]
mod lex;
mod parse;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result};

#[derive(Clone)]
pub struct Located<T> {
    row: usize,
    col: usize,
    value: T,
}

impl<T> Located<T> {
    pub fn new(row: usize, col: usize, value: T) -> Self {
        Self { row, col, value }
    }

    pub fn row(&self) -> usize {
        self.row
    }

    pub fn col(&self) -> usize {
        self.col
    }

    pub fn value(&self) -> &T {
        &self.value
    }
}

impl<E: Error> Debug for Located<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}:{}: {}", self.row, self.col, self.value)
    }
}

impl<E: Error> Display for Located<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        <Self as Debug>::fmt(&self, f)
    }
}

impl<E: Error> Error for Located<E> {}

mod error {
    use std::error::Error;
    use std::fmt::{Display, Formatter, Result};

    #[derive(Debug, Clone)]
    pub enum Syntax {
        UnterminatedString,
        StrayCharacter(char),
    }

    impl Display for Syntax {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            match self {
                Self::UnterminatedString => write!(f, "missing terminating '\"' character"),
                Self::StrayCharacter(c) => write!(f, "stray {} in program", c),
            }
        }
    }

    impl Error for Syntax {}
}
