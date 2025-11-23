use std::fmt;

use crate::token::*;

#[derive(Debug)]
pub enum LexError {
    UnexepectedChar { ch: char, span: (usize, usize) },
    InvalidNumber { lexeme: String, span: (usize, usize) },
    InvalidUTF8 { span: (usize, usize) },
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // This variant has a char and a span
            LexError::UnexepectedChar { ch, span } => {
                write!(f, "Unexpected character '{}' at {}..{}", ch, span.0, span.1)
            }
            // This variant has a string lexeme and a span
            LexError::InvalidNumber { lexeme, span } => {
                write!(f, "Invalid number '{}' at {}..{}", lexeme, span.0, span.1)
            }
            // This variant has only a span
            LexError::InvalidUTF8 { span } => {
                write!(f, "Invalid UTF-8 sequence at {}..{}", span.0, span.1)
            }
        }
    }
}

pub struct Lexer<'a> {
    source: &'a [u8],
    tokens: Vec<Token>,
    start: usize,
    curr: usize
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source.as_bytes(),
            tokens: Vec::new(),
            start: 0,
            curr: 0
        }
    }

    pub fn scan(&mut self) -> Result<&Vec<Token>, LexError> {
        while !self.is_at_end() {
            self.start = self.curr;
            let c = self.advance();
            match c {
                '(' => self.add_token(TokenType::LeftParen, c),
                ')' => self.add_token(TokenType::RightParen, c),
                ',' => self.add_token(TokenType::Comma, c),
                '-' => self.add_token(TokenType::Minus, c),
                '+' => self.add_token(TokenType::Plus, c),
                '*' => self.add_token(TokenType::Star, c),
                '/' => self.add_token(TokenType::Slash, c),

                ' ' | '\r' | '\n' | '\t' => {},

                '0'..='9' | '.' => match self.number() {
                    Err(e) => { return Err(e); },
                    _ => {},
                    
                },
                'a'..'z' | 'A'..='Z' => match self.identifier() {
                    Err(e) => { return Err(e); },
                    _ => {},
                },
                _ => {
                    return Err(LexError::UnexepectedChar { ch: c, span: (self.curr, self.start) });
                }
            }
        }

        self.add_token(TokenType::EOF, "");
        Ok(&self.tokens)
    }

    fn add_token<S : Into<String>>(&mut self, token_type: TokenType, lexeme: S) {
        self.tokens.push(Token {
            token_type,
            lexeme: lexeme.into(),
            span: (self.start, self.curr),
        })
    }

    fn identifier(&mut self) -> Result<(), LexError> {
        while Self::is_alpha(self.peek()) {
            self.advance();
        }

        let lexeme = str::from_utf8(&self.source[self.start..self.curr])
            .map_err(|_| LexError::InvalidUTF8 { span: (self.start, self.curr) })?;

        self.add_token(TokenType::Identifier, lexeme);
        Ok(())
    }

    fn number(&mut self) -> Result<(), LexError> {
        while Self::is_digit(self.peek()) {
            self.advance();
        }

        // Optional decimal part
        if self.peek() == '.' && Self::is_digit(self.peek_next()) {
            self.advance(); // consume '.'
            while Self::is_digit(self.peek()) || self.peek() == '.' {
                self.advance();
            }
        }

        let lexeme = str::from_utf8(&self.source[self.start..self.curr])
            .map_err(|_| LexError::InvalidUTF8 { span: (self.start, self.curr) })?;

        if lexeme.parse::<f64>().is_err() {
            return Err(LexError::InvalidNumber {
                lexeme: lexeme.to_string(),
                span: (self.start, self.curr),
            });
        }

        self.add_token(TokenType::Number, lexeme);
        Ok(())
    }

    fn is_digit(c: char) -> bool {
        '0' <= c && c <= '9'
    }

    fn is_alpha(c: char) -> bool {
        'a' <= c && c <= 'z' || 'A' <= c && c <= 'Z'
    }

    fn peek(&mut self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        return self.source[self.curr] as char;
    }

    fn peek_next(&mut self) -> char {
        let index = self.curr + 1;
        if index >= self.source.len() {
            return '\0';
        }
        return self.source[index] as char;
    }

    fn advance(&mut self) -> char {
        let res = self.source[self.curr] as char;
        self.curr += 1;
        return res;
    }

    fn is_at_end(&self) -> bool {
        self.curr >= self.source.len()
    }

}
