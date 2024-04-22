mod chunk;
mod lex;
mod vm;
mod compiler;

#[derive(Clone, Debug)]
enum Position {
    Coords(usize, usize),
    Eof,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum CompileError {
    UnclosedString,
    StrayChar(char),
}
