use crate::location::{AtCoords, Coords};

#[derive(Clone, Copy, Debug, PartialOrd, PartialEq, Eq)]
pub enum Kind {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    String,
    Number,
    Identifier,
}

#[derive(Clone)]
pub struct Token<'a> {
    kind: Kind,
    span: &'a str,
}

impl<'a> Token<'a> {
    pub fn new(kind: Kind, span: &'a str) -> Self {
        Self { kind, span }
    }
}

impl AtCoords<Token<'_>> {
    pub fn kind(&self) -> Kind {
        self.value.kind.clone()
    }

    pub fn span(&self) -> &str {
        self.value.span
    }
}
