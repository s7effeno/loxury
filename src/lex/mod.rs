use std::{iter::Peekable, str::Chars};

mod token;
pub use token::Token;
use token::KEYWORDS;
mod error;

#[derive(Clone)]
struct Text<'a> {
    row: usize,
    col: usize,
    text: Peekable<Chars<'a>>,
}

impl<'a> Text<'a> {
    fn new(s: &'a str) -> Self {
        Self {
            row: 1,
            col: 1,
            text: s.chars().peekable(),
        }
    }

    fn row(&self) -> usize {
        self.row
    }

    fn col(&self) -> usize {
        self.col
    }

    fn peek(&mut self) -> Option<&<Self as Iterator>::Item> {
        self.text.peek()
    }

    fn next_if(
        &mut self,
        func: impl FnOnce(&<Self as Iterator>::Item) -> bool,
    ) -> Option<<Self as Iterator>::Item> {
        self.peek().copied().filter(|c| func(c)).map(|c| {
            self.next();
            c
        })
    }

    fn peeking_take_string_while<F>(&mut self, accept: F) -> String
    where
        Self: Sized,
        F: FnMut(&<Self as Iterator>::Item) -> bool,
    {
        let mut accept = accept;
        let mut ret = String::new();
        while let Some(c) = self.next_if(&mut accept) {
            ret.push(c);
        }
        ret
    }
}

impl Iterator for Text<'_> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match self.text.next() {
            Some(c) => {
                if c == '\n' {
                    self.col = 1;
                    self.row += 1;
                } else {
                    self.col += 1
                }
                Some(c)
            }
            None => None,
        }
    }
}

pub struct Lexer<'a> {
    row: usize,
    col: usize,
    source: Text<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            row: 1,
            col: 1,
            source: Text::new(source),
        }
    }

    fn local<T>(&self, data: T) -> (usize, usize, T) {
        (self.row, self.col, data)
    }

    // TODO move these two methods to Token and Error
    fn local_token(&self, token: Token) -> (usize, usize, Result<Token, ()>) {
        self.local(Ok(token))
    }

    fn local_err(&self, err: ()) -> (usize, usize, Result<Token, ()>) {
        self.local(Err(err))
    }

    fn integer(&mut self) -> String {
        self.source
            .by_ref()
            .peeking_take_string_while(|c| c.is_numeric())
    }
}

impl Iterator for Lexer<'_> {
    // TODO: change to error
    type Item = (usize, usize, Result<Token, ()>);

    fn next(&mut self) -> Option<Self::Item> {
        Some({
            self.row = self.source.row();
            self.col = self.source.col();

            match self.source.peek()? {
                '(' => {
                    self.source.next();
                    self.local_token(Token::LeftParen)
                }
                ')' => {
                    self.source.next();
                    self.local_token(Token::RightParen)
                }
                '{' => {
                    self.source.next();
                    self.local_token(Token::LeftBrace)
                }
                '}' => {
                    self.source.next();
                    self.local_token(Token::RightBrace)
                }
                ',' => {
                    self.source.next();
                    self.local_token(Token::Comma)
                }
                '.' => {
                    self.source.next();
                    self.local_token(Token::Dot)
                }
                '-' => {
                    self.source.next();
                    self.local_token(Token::Minus)
                }
                '+' => {
                    self.source.next();
                    self.local_token(Token::Plus)
                }
                ';' => {
                    self.source.next();
                    self.local_token(Token::Semicolon)
                }
                '*' => {
                    self.source.next();
                    self.local_token(Token::Star)
                }
                '/' => {
                    self.source.next();
                    match self.source.peek() {
                        Some('/') => {
                            self.source
                                .by_ref()
                                .take_while(|c| *c != '\n')
                                .for_each(drop);
                            self.next()?
                        }
                        _ => self.local_token(Token::Slash),
                    }
                }
                '!' => {
                    self.source.next();
                    let token = self
                        .source
                        .next_if(|c| *c == '=')
                        .map_or(Token::Bang, |_| Token::BangEqual);
                    self.local_token(token)
                }
                '=' => {
                    self.source.next();
                    let token = self
                        .source
                        .next_if(|c| *c == '=')
                        .map_or(Token::Equal, |_| Token::EqualEqual);
                    self.local_token(token)
                }
                '>' => {
                    self.source.next();
                    let token = self
                        .source
                        .next_if(|c| *c == '=')
                        .map_or(Token::Greater, |_| Token::GreaterEqual);
                    self.local_token(token)
                }
                '<' => {
                    self.source.next();
                    let token = self
                        .source
                        .next_if(|c| *c == '=')
                        .map_or(Token::Less, |_| Token::LessEqual);
                    self.local_token(token)
                }
                '"' => {
                    self.source.next();
                    let str = self
                        .source
                        .by_ref()
                        .peeking_take_string_while(|c| *c != '"');
                    // TODO: move collect inside if-let
                    if let Some('"') = self.source.next() {
                        self.local_token(Token::String(str))
                    } else {
                        self.local_err(())
                    }
                }
                c if c.is_numeric() => {
                    let mut number = self.integer();
                    let mut cloned = self.source.clone();
                    if let (Some('.'), Some(c)) = (cloned.next(), cloned.next()) {
                        if c.is_numeric() {
                            number.push('.');
                            // remove '.'
                            self.source.next();
                            number.push_str(&self.integer());
                        }
                    }
                    self.local_token(Token::Number(number.parse().unwrap()))
                }
                c if c.is_alphabetic() || *c == '_' => {
                    let identifier: String = self
                        .source
                        .peeking_take_string_while(|c| c.is_alphanumeric() || *c == '_');
                    self.local_token(
                        KEYWORDS
                            .get(&*identifier)
                            .map_or(Token::Identifier(identifier), |t| t.clone()),
                    )
                }
                c if c.is_whitespace() => {
                    self.source.next();
                    self.next()?
                }
                _ => self.local_err(()),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn left_paren() {
        assert!(matches!(
            Lexer::new("(").next().unwrap().data(),
            Ok(Token::LeftParen)
        ))
    }

    #[test]
    fn right_paren() {
        assert!(matches!(
            Lexer::new(")").next().unwrap().data(),
            Ok(Token::RightParen)
        ))
    }

    #[test]
    fn left_brace() {
        assert!(matches!(
            Lexer::new("{").next().unwrap().data(),
            Ok(Token::LeftBrace)
        ))
    }

    #[test]
    fn right_brace() {
        assert!(matches!(
            Lexer::new("}").next().unwrap().data(),
            Ok(Token::RightBrace)
        ))
    }

    #[test]
    fn comma() {
        assert!(matches!(
            Lexer::new(",").next().unwrap().data(),
            Ok(Token::Comma)
        ))
    }

    #[test]
    fn dot() {
        assert!(matches!(
            Lexer::new(".").next().unwrap().data(),
            Ok(Token::Dot)
        ))
    }

    #[test]
    fn minus() {
        assert!(matches!(
            Lexer::new("-").next().unwrap().data(),
            Ok(Token::Minus)
        ))
    }

    #[test]
    fn plus() {
        assert!(matches!(
            Lexer::new("+").next().unwrap().data(),
            Ok(Token::Plus)
        ))
    }

    #[test]
    fn semicolon() {
        assert!(matches!(
            Lexer::new(";").next().unwrap().data(),
            Ok(Token::Semicolon)
        ))
    }

    #[test]
    fn slash() {
        assert!(matches!(
            Lexer::new("/").next().unwrap().data(),
            Ok(Token::Slash)
        ))
    }

    #[test]
    fn star() {
        assert!(matches!(
            Lexer::new("*").next().unwrap().data(),
            Ok(Token::Star)
        ))
    }

    #[test]
    fn bang() {
        assert!(matches!(
            Lexer::new("!").next().unwrap().data(),
            Ok(Token::Bang)
        ))
    }

    #[test]
    fn bang_equal() {
        assert!(matches!(
            Lexer::new("!=").next().unwrap().data(),
            Ok(Token::BangEqual)
        ))
    }

    #[test]
    fn equal() {
        assert!(matches!(
            Lexer::new("=").next().unwrap().data(),
            Ok(Token::Equal)
        ))
    }

    #[test]
    fn equal_equal() {
        assert!(matches!(
            Lexer::new("==").next().unwrap().data(),
            Ok(Token::EqualEqual)
        ))
    }

    #[test]
    fn greater() {
        assert!(matches!(
            Lexer::new(">").next().unwrap().data(),
            Ok(Token::Greater)
        ))
    }

    #[test]
    fn greater_equal() {
        assert!(matches!(
            Lexer::new(">=").next().unwrap().data(),
            Ok(Token::GreaterEqual)
        ))
    }

    #[test]
    fn less() {
        assert!(matches!(
            Lexer::new("<").next().unwrap().data(),
            Ok(Token::Less)
        ))
    }

    #[test]
    fn less_equal() {
        assert!(matches!(
            Lexer::new("<=").next().unwrap().data(),
            Ok(Token::LessEqual)
        ))
    }

    #[test]
    fn and() {
        assert!(matches!(
            Lexer::new("and").next().unwrap().data(),
            Ok(Token::And)
        ))
    }

    #[test]
    fn class() {
        assert!(matches!(
            Lexer::new("class").next().unwrap().data(),
            Ok(Token::Class)
        ))
    }

    #[test]
    fn r#else() {
        assert!(matches!(
            Lexer::new("else").next().unwrap().data(),
            Ok(Token::Else)
        ))
    }

    #[test]
    fn r#false() {
        assert!(matches!(
            Lexer::new("false").next().unwrap().data(),
            Ok(Token::False)
        ))
    }

    #[test]
    fn fun() {
        assert!(matches!(
            Lexer::new("fun").next().unwrap().data(),
            Ok(Token::Fun)
        ))
    }

    #[test]
    fn r#for() {
        assert!(matches!(
            Lexer::new("for").next().unwrap().data(),
            Ok(Token::For)
        ))
    }

    #[test]
    fn r#if() {
        assert!(matches!(
            Lexer::new("if").next().unwrap().data(),
            Ok(Token::If)
        ))
    }

    #[test]
    fn nil() {
        assert!(matches!(
            Lexer::new("nil").next().unwrap().data(),
            Ok(Token::Nil)
        ))
    }

    #[test]
    fn or() {
        assert!(matches!(
            Lexer::new("or").next().unwrap().data(),
            Ok(Token::Or)
        ))
    }
    #[test]
    fn print() {
        assert!(matches!(
            Lexer::new("print").next().unwrap().data(),
            Ok(Token::Print)
        ))
    }

    #[test]
    fn r#return() {
        assert!(matches!(
            Lexer::new("return").next().unwrap().data(),
            Ok(Token::Return)
        ))
    }

    #[test]
    fn ssuper() {
        assert!(matches!(
            Lexer::new("super").next().unwrap().data(),
            Ok(Token::Super)
        ))
    }

    #[test]
    fn this() {
        assert!(matches!(
            Lexer::new("this").next().unwrap().data(),
            Ok(Token::This)
        ))
    }

    #[test]
    fn r#true() {
        assert!(matches!(
            Lexer::new("true").next().unwrap().data(),
            Ok(Token::True)
        ))
    }

    #[test]
    fn r#var() {
        assert!(matches!(
            Lexer::new("var").next().unwrap().data(),
            Ok(Token::Var)
        ))
    }

    #[test]
    fn r#while() {
        assert!(matches!(
            Lexer::new("while").next().unwrap().data,
            Ok(Token::While)
        ))
    }

    #[test]
    fn string() {
        if let Ok(Token::String(s)) = Lexer::new("\"AAAABBBB00001111\"").next().unwrap().data() {
            assert_eq!("AAAABBBB00001111", s);
        } else {
            panic!()
        }
    }

    #[test]
    fn number() {
        if let Ok(Token::Number(n)) = Lexer::new("12.34").next().unwrap().data() {
            assert_eq!(12.34, n);
        } else {
            panic!()
        }
    }

    #[test]
    fn comment() {
        assert!(matches!(Lexer::new("//").next(), None))
    }
}
