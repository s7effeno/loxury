#![feature(test)]
mod lex;
mod parse;

#[derive(Debug)]
pub struct Local<T> {
    row: usize,
    col: usize,
    data: T,
}

impl<T> Local<T> {
    pub fn new(row: usize, col: usize, data: T) -> Self {
        Self { row, col, data }
    }

    pub fn data(self) -> T {
        self.data
    }
}
