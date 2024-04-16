// use crate::error::Syntax as SyntaxError;
use crate::CompileError;
use crate::Located;
use std::str::Chars;

mod token;
pub use token::Token;

#[derive(Clone)]
struct Text<'a> {
    row: usize,
    col: usize,
    text: Chars<'a>,
}

impl<'a> Text<'a> {
    fn new(s: &'a str) -> Self {
        Self {
            row: 1,
            col: 1,
            text: s.chars(),
        }
    }

    fn row(&self) -> usize {
        self.row
    }

    fn col(&self) -> usize {
        self.col
    }

    fn peek(&self) -> Option<char> {
        self.text.clone().next()
    }

    fn next_if(&mut self, func: impl FnOnce(char) -> bool) -> Option<char> {
        self.peek().filter(|c| func(*c)).map(|c| {
            self.next();
            c
        })
    }

    fn advance_while<'b, F>(&'b mut self, accept: F) -> usize
    where
        Self: Sized,
        F: Fn(char) -> bool,
    {
        let mut end = 0;
        while let Some(c) = self.next_if(&accept) {
            end += c.len_utf8();
        }
        end
    }

    fn take_str_while<'b, F>(&'b mut self, accept: F) -> &'a str
    where
        Self: Sized,
        F: Fn(char) -> bool,
    {
        &self.text.as_str()[..self.advance_while(accept)]
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

    fn located<T>(&self, data: T) -> Located<T> {
        Located::at_coords(self.row, self.col, data)
    }

    fn local_token(
        &'a self,
        token: Token<'a>,
    ) -> Result<Located<Token<'a>>, Located<CompileError>> {
        Ok(self.located(token))
    }

    fn local_err(&'a self, err: CompileError) -> Result<Located<Token<'a>>, Located<CompileError>> {
        Err(self.located(err))
    }

    fn next<'b>(&'b mut self) -> Option<Result<Located<Token<'b>>, Located<CompileError>>> {
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
                    if self.source.next_if(|c| c == '/').is_some() {
                        self.source
                            .by_ref()
                            .take_while(|c| *c != '\n')
                            .for_each(drop);
                        self.next()?
                    } else {
                        self.local_token(Token::Slash)
                    }
                }
                '!' => {
                    self.source.next();
                    let token = self
                        .source
                        .next_if(|c| c == '=')
                        .map_or(Token::Bang, |_| Token::BangEqual);
                    self.local_token(token)
                }
                '=' => {
                    self.source.next();
                    let token = self
                        .source
                        .next_if(|c| c == '=')
                        .map_or(Token::Equal, |_| Token::EqualEqual);
                    self.local_token(token)
                }
                '>' => {
                    self.source.next();
                    let token = self
                        .source
                        .next_if(|c| c == '=')
                        .map_or(Token::Greater, |_| Token::GreaterEqual);
                    self.local_token(token)
                }
                '<' => {
                    self.source.next();
                    let token = self
                        .source
                        .next_if(|c| c == '=')
                        .map_or(Token::Less, |_| Token::LessEqual);
                    self.local_token(token)
                }
                '"' => {
                    self.source.next();
                    let s = self.source.take_str_while(|c| c != '"');
                    if self.source.next().is_some() {
                        self.local_token(Token::String(s))
                    } else {
                        Err(Located::at_eof(CompileError::UnclosedString))
                    }
                }
                c if c.is_numeric() => {
                    let start = self.source.text.as_str();
                    let mut len = self.source.advance_while(|c| c.is_numeric());
                    let mut cloned = self.source.text.clone();
                    if cloned.next().is_some_and(|c| c == '.')
                        && cloned.next().is_some_and(|c| c.is_numeric())
                    {
                        // remove '.'
                        self.source.next();
                        len += 1 + self.source.advance_while(|c| c.is_numeric());
                    }
                    self.local_token(Token::Number(start[..len].parse().unwrap()))
                }
                c if c.is_alphabetic() => {
                    let identifier = self.source.take_str_while(|c| c.is_alphabetic());

                    fn map_identifier(identifier: &str) -> Token {
                        let mut iter = identifier.bytes();
                        let (rest, token) = match iter.next().unwrap() {
                            b'a' => ("nd", Token::And),
                            b'c' => ("lass", Token::Class),
                            b'e' => ("lse", Token::Else),
                            b'i' => ("f", Token::If),
                            b'n' => ("il", Token::Nil),
                            b'o' => ("r", Token::Or),
                            b'p' => ("rint", Token::Print),
                            b'r' => ("eturn", Token::Return),
                            b's' => ("uper", Token::Super),
                            b'v' => ("ar", Token::Var),
                            b'w' => ("hile", Token::While),
                            b'f' => {
                                match iter.next() {
                                    Some(b'a') => ("lse", Token::False),
                                    Some(b'o') => ("r", Token::For),
                                    Some(b'u') => ("n", Token::Fun),
                                    _ => return Token::Identifier(identifier),
                                }
                            }
                            b't' => {
                                match iter.next() {
                                    Some(b'h') => ("is", Token::This),
                                    Some(b'r') => ("ue", Token::True),
                                    _ => return Token::Identifier(identifier),                            }
                            }
                            _ => return Token::Identifier(identifier),
                        };
                        
                        if iter.eq(rest.bytes()) {
                            token
                        } else {
                            Token::Identifier(identifier)
                        }
                    }

                    self.local_token(map_identifier(identifier))
                }
                c if c.is_whitespace() => {
                    while self.source.next_if(|c| c.is_whitespace()).is_some() {
                        self.source.next();
                    }
                    self.next()?
                }
                c => {
                    self.source.next();
                    self.local_err(CompileError::StrayChar(c))
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn big_program() {
        use std::time::Instant;
        let source = include_str!("input");
        let mut lexer = Lexer::new(source);
        let before = Instant::now();
        while let Some(a) = lexer.next() {
            drop(a);
        }
        let elapsed = before.elapsed();
        println!("{:?}", elapsed.as_millis());
    }

    #[test]
    fn left_paren() {
        assert!(matches!(
            Lexer::new("(").next().unwrap().unwrap().value(),
            Token::LeftParen
        ))
    }

    #[test]
    fn right_paren() {
        assert!(matches!(
            Lexer::new(")").next().unwrap().unwrap().value(),
            Token::RightParen
        ))
    }

    #[test]
    fn left_brace() {
        assert!(matches!(
            Lexer::new("{").next().unwrap().unwrap().value(),
            Token::LeftBrace
        ))
    }

    #[test]
    fn right_brace() {
        assert!(matches!(
            Lexer::new("}").next().unwrap().unwrap().value(),
            Token::RightBrace
        ))
    }

    #[test]
    fn comma() {
        assert!(matches!(
            Lexer::new(",").next().unwrap().unwrap().value(),
            Token::Comma
        ))
    }

    #[test]
    fn dot() {
        assert!(matches!(
            Lexer::new(".").next().unwrap().unwrap().value(),
            Token::Dot
        ))
    }

    #[test]
    fn minus() {
        assert!(matches!(
            Lexer::new("-").next().unwrap().unwrap().value(),
            Token::Minus
        ))
    }

    #[test]
    fn plus() {
        assert!(matches!(
            Lexer::new("+").next().unwrap().unwrap().value(),
            Token::Plus
        ))
    }

    #[test]
    fn semicolon() {
        assert!(matches!(
            Lexer::new(";").next().unwrap().unwrap().value(),
            Token::Semicolon
        ))
    }

    #[test]
    fn slash() {
        assert!(matches!(
            Lexer::new("/").next().unwrap().unwrap().value(),
            Token::Slash
        ))
    }

    #[test]
    fn star() {
        assert!(matches!(
            Lexer::new("*").next().unwrap().unwrap().value(),
            Token::Star
        ))
    }

    #[test]
    fn bang() {
        assert!(matches!(
            Lexer::new("!").next().unwrap().unwrap().value(),
            Token::Bang
        ))
    }

    #[test]
    fn bang_equal() {
        assert!(matches!(
            Lexer::new("!=").next().unwrap().unwrap().value(),
            Token::BangEqual
        ))
    }

    #[test]
    fn equal() {
        assert!(matches!(
            Lexer::new("=").next().unwrap().unwrap().value(),
            Token::Equal
        ))
    }

    #[test]
    fn equal_equal() {
        assert!(matches!(
            Lexer::new("==").next().unwrap().unwrap().value(),
            Token::EqualEqual
        ))
    }

    #[test]
    fn greater() {
        assert!(matches!(
            Lexer::new(">").next().unwrap().unwrap().value(),
            Token::Greater
        ))
    }

    #[test]
    fn greater_equal() {
        assert!(matches!(
            Lexer::new(">=").next().unwrap().unwrap().value(),
            Token::GreaterEqual
        ))
    }

    #[test]
    fn less() {
        assert!(matches!(
            Lexer::new("<").next().unwrap().unwrap().value(),
            Token::Less
        ))
    }

    #[test]
    fn less_equal() {
        assert!(matches!(
            Lexer::new("<=").next().unwrap().unwrap().value(),
            Token::LessEqual
        ))
    }

    #[test]
    fn and() {
        assert!(matches!(
            Lexer::new("and").next().unwrap().unwrap().value(),
            Token::And
        ))
    }

    #[test]
    fn class() {
        assert!(matches!(
            Lexer::new("class").next().unwrap().unwrap().value(),
            Token::Class
        ))
    }

    #[test]
    fn r#else() {
        assert!(matches!(
            Lexer::new("else").next().unwrap().unwrap().value(),
            Token::Else
        ))
    }

    #[test]
    fn r#false() {
        assert!(matches!(
            Lexer::new("false").next().unwrap().unwrap().value(),
            Token::False
        ))
    }

    #[test]
    fn fun() {
        assert!(matches!(
            Lexer::new("fun").next().unwrap().unwrap().value(),
            Token::Fun
        ))
    }

    #[test]
    fn r#for() {
        assert!(matches!(
            Lexer::new("for").next().unwrap().unwrap().value(),
            Token::For
        ))
    }

    #[test]
    fn r#if() {
        assert!(matches!(
            Lexer::new("if").next().unwrap().unwrap().value(),
            Token::If
        ))
    }

    #[test]
    fn nil() {
        assert!(matches!(
            Lexer::new("nil").next().unwrap().unwrap().value(),
            Token::Nil
        ))
    }

    #[test]
    fn or() {
        assert!(matches!(
            Lexer::new("or").next().unwrap().unwrap().value(),
            Token::Or
        ))
    }

    #[test]
    fn print() {
        assert!(matches!(
            Lexer::new("print").next().unwrap().unwrap().value(),
            Token::Print
        ))
    }

    #[test]
    fn r#return() {
        assert!(matches!(
            Lexer::new("return").next().unwrap().unwrap().value(),
            Token::Return
        ))
    }

    #[test]
    fn ssuper() {
        assert!(matches!(
            Lexer::new("super").next().unwrap().unwrap().value(),
            Token::Super
        ))
    }

    #[test]
    fn this() {
        assert!(matches!(
            Lexer::new("this").next().unwrap().unwrap().value(),
            Token::This
        ))
    }

    #[test]
    fn r#true() {
        assert!(matches!(
            Lexer::new("true").next().unwrap().unwrap().value(),
            Token::True
        ))
    }

    #[test]
    fn r#var() {
        assert!(matches!(
            Lexer::new("var").next().unwrap().unwrap().value(),
            Token::Var
        ))
    }

    #[test]
    fn r#while() {
        assert!(matches!(
            Lexer::new("while").next().unwrap().unwrap().value(),
            Token::While
        ))
    }

    #[test]
    fn string() {
        if let Token::String(s) = Lexer::new("\"AAAABBBB00001111\"")
            .next()
            .unwrap()
            .unwrap()
            .value()
        {
            assert_eq!(&"AAAABBBB00001111", s);
        } else {
            panic!()
        }
    }

    #[test]
    fn number() {
        if let Token::Number(n) = Lexer::new("12.34").next().unwrap().unwrap().value() {
            assert_eq!(12.34, *n);
        } else {
            panic!()
        }
    }

    #[test]
    fn comment() {
        assert!(matches!(Lexer::new("//").next(), None))
    }

    #[test]
    fn unterminated_string() {
        assert!(matches!(
            Lexer::new("\"").next().unwrap().err().unwrap().value(),
            CompileError::UnclosedString
        ))
    }

    #[test]
    fn stray_character() {
        assert!(matches!(
            Lexer::new("`").next().unwrap().err().unwrap().value(),
            CompileError::StrayChar('`')
        ))
    }
}
