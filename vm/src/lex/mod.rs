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
            println!("LKJSFDLJKSD {c}");
            end += c.len_utf8();
        }
        end
    }

    fn take_str_while<'b, F>(&'b mut self, accept: F) -> &'a str
    where
        Self: Sized,
        F: Fn(char) -> bool,
    {
        println!("YOOO: {}", self.text.as_str());
        &self.text.as_str()[..=self.advance_while(accept)]
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

    fn integer(&mut self) -> &str {
        todo!()
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

    fn next(&'a mut self) -> Option<Result<Located<Token<'a>>, Located<CompileError>>> {
        Some({
            self.row = self.source.row();
            self.col = self.source.col();

            if self.source.next_if(|c| c == '(').is_some() {
                self.local_token(Token::LeftParen)
            } else if self.source.next_if(|c| c == ')').is_some() {
                self.local_token(Token::RightParen)
            } else if self.source.next_if(|c| c == '{').is_some() {
                self.local_token(Token::LeftBrace)
            } else if self.source.next_if(|c| c == '}').is_some() {
                self.local_token(Token::RightBrace)
            } else if self.source.next_if(|c| c == ',').is_some() {
                self.local_token(Token::Comma)
            } else if self.source.next_if(|c| c == '.').is_some() {
                self.local_token(Token::Dot)
            } else if self.source.next_if(|c| c == '-').is_some() {
                self.local_token(Token::Minus)
            } else if self.source.next_if(|c| c == '+').is_some() {
                self.local_token(Token::Plus)
            } else if self.source.next_if(|c| c == ';').is_some() {
                self.local_token(Token::Semicolon)
            } else if self.source.next_if(|c| c == '*').is_some() {
                self.local_token(Token::Star)
            } else if self.source.next_if(|c| c == '/').is_some() {
                if self.source.next_if(|c| c == '/').is_some() {
                    self.source
                        .by_ref()
                        .take_while(|c| *c != '\n')
                        .for_each(drop);
                    self.next()?
                } else {
                    self.local_token(Token::Slash)
                }
            } else if self.source.next_if(|c| c == '!').is_some() {
                let token = self
                    .source
                    .next_if(|c| c == '=')
                    .map_or(Token::Bang, |_| Token::BangEqual);
                self.local_token(token)
            } else if self.source.next_if(|c| c == '=').is_some() {
                let token = self
                    .source
                    .next_if(|c| c == '=')
                    .map_or(Token::Equal, |_| Token::EqualEqual);
                self.local_token(token)
            } else if self.source.next_if(|c| c == '>').is_some() {
                let token = self
                    .source
                    .next_if(|c| c == '=')
                    .map_or(Token::Greater, |_| Token::GreaterEqual);
                self.local_token(token)
            } else if self.source.next_if(|c| c == '<').is_some() {
                let token = self
                    .source
                    .next_if(|c| c == '=')
                    .map_or(Token::Less, |_| Token::LessEqual);
                self.local_token(token)
            } else if self.source.next_if(|c| c == '"').is_some() {
                let s = self.source.take_str_while(|c| c != '"');
                if let Some('"') = self.source.next() {
                    self.local_token(Token::String(s))
                } else {
                    Err(Located::at_eof(CompileError::UnclosedString))
                }
            } else if self.source.peek().is_some_and(|c| c.is_numeric()) {
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
            } else if self.source.peek().is_some_and(|c| c.is_alphabetic()) {
                todo!()
            } else if self.source.next_if(|c| c.is_whitespace()).is_some() {
                self.next()?
            } else {
                let c = self.source.next()?;
                self.local_err(CompileError::StrayChar(c))
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
