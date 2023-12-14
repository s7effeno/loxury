#![feature(test)]
mod interpret;
mod lex;
mod parse;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result};

#[derive(Clone)]
enum Position {
    Coords(usize, usize),
    Eof,
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
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

impl<E: Error> Debug for Located<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}: {:?}", self.pos, self.value)
    }
}

impl<E: Error> Display for Located<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}: {}", self.pos, self.value)
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
        ExpectedExpression,
        UnclosedGrouping,
        UnterminatedExprStatement,
    }

    impl Display for Syntax {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            match self {
                Self::UnterminatedString => write!(f, "expected '\"' at the end of string"),
                Self::StrayCharacter(c) => write!(f, "stray {} in program", c),
                Self::ExpectedExpression => write!(f, "expected expression"),
                Self::UnclosedGrouping => {
                    write!(f, "expected ')' at the end of grouping expression")
                }
                Self::UnterminatedExprStatement => {
                    write!(f, "expected ';' at the end of statement")
                }
            }
        }
    }

    impl Error for Syntax {}
}
