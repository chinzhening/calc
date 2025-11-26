use std::fmt;

use crate::token::*;



pub fn scan<T: AsRef<[u8]>>(source: T) -> Result<Vec<Token>, LexError> {
    let mut lexer = Lexer::from_bytes(source.as_ref());
    lexer.scan().cloned()
}






#[derive(Debug, PartialEq)]
pub enum LexError {
    UnexpectedChar { char: String, span: (usize, usize) },
    UnknownIdentifier { lexeme: String, span: (usize, usize) },
    InvalidNumber { lexeme: String, span: (usize, usize) },
    InvalidUTF8 { span: (usize, usize) },
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexError::UnexpectedChar { char, span } => {
                write!(f, "Unexpected character '{}' at {}..{}", char, span.0, span.1)
            }
            LexError::UnknownIdentifier { lexeme, span } => {
                write!(f, "Unknown identifier '{}' at {}..{}", lexeme, span.0, span.1)
            }
            LexError::InvalidNumber { lexeme, span } => {
                write!(f, "Invalid number '{}' at {}..{}", lexeme, span.0, span.1)
            }
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
    pub fn from_bytes(source: &'a [u8]) -> Self {
        Self {
            source,
            tokens: Vec::new(),
            start: 0,
            curr: 0
        }
    }

    pub fn from_str(source: &'a str) -> Self {
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
                    return Err(LexError::UnexpectedChar {
                        char: c.to_string(), span: (self.start, self.curr)
                    });
                }
            }
        }

        self.start = self.curr;
        self.increment();
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
            .map_err(|_| LexError::InvalidUTF8 { span: (self.start, self.curr) }
        )?;

        let token_type: TokenType = self.identifier_type(lexeme)?;

        self.add_token(token_type, lexeme);
        Ok(())
    }

    fn identifier_type(&mut self, lexeme: &str) -> Result<TokenType, LexError> {
        match lexeme {
            "sin" => Ok(TokenType::Sin),
            "cos" => Ok(TokenType::Cos),
            "tan" => Ok(TokenType::Tan),
            "arcsin" => Ok(TokenType::ArcSin),
            "arccos" => Ok(TokenType::ArcCos),
            "arctan" => Ok(TokenType::ArcTan),
            _ => Err(LexError::UnknownIdentifier { lexeme: lexeme.into(), span: (self.start, self.curr) }),
        }
    } 

    fn number(&mut self) -> Result<(), LexError> {
        while Self::is_digit(self.peek()) {
            let c = self.advance();
            println!("{}", c);
        }

        // Optional decimal part
        if self.peek() == '.' {
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

    fn advance(&mut self) -> char {
        let res = self.source[self.curr] as char;
        self.increment();
        return res;
    }

    fn is_at_end(&self) -> bool {
        self.curr >= self.source.len()
    }

    fn increment(&mut self) {
        self.curr += 1;
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    fn make_token<S : Into<String>>(token_type: TokenType, lexeme: S, span: (usize, usize)) -> Token {
        Token { token_type, lexeme: lexeme.into(), span }
    }

    fn assert_lex(input: &str, expected: &Vec<Token>) {
        let mut lexer = Lexer::from_str(input);
        let tokens = lexer.scan().unwrap();
        assert_eq!(tokens, expected);
    }

    fn assert_lex_error(input: &str, expected: LexError) {
        let mut lexer = Lexer::from_str(input);
        let result = lexer.scan();
        match result {
            Ok(tokens) => panic!("Expected error {:?}, but got Ok: {:?}", expected, tokens),
            Err(e) => assert_eq!(e, expected)
        }
    }

    #[test]
    fn test_eof_with_crlf() {
        assert_lex("\r\n",&vec![make_token(TokenType::EOF, "", (2, 3))]);
    }

    #[test]
    fn test_eof_single_whitespace() {
        assert_lex(" ", &vec![make_token(TokenType::EOF, "", (1, 2))]);
    }

    #[test]
    fn test_eof_multiple_whitespace() {
        assert_lex("   ", &vec![make_token(TokenType::EOF, "", (3, 4))]);
    }

    #[test]
    fn test_number_valid() {
        assert_lex(
            ".1",
            &vec![
                make_token(TokenType::Number, ".1", (0, 2)),
                make_token(TokenType::EOF, "", (2, 3)),
            ]
        );
        assert_lex(
            "1.",
            &vec![
                make_token(TokenType::Number, "1.", (0, 2)),
                make_token(TokenType::EOF, "", (2, 3)),
            ]
        );
        assert_lex(
            "1.1",
            &vec![
                make_token(TokenType::Number, "1.1", (0, 3)),
                make_token(TokenType::EOF, "", (3, 4)),
            ]
        );
        assert_lex(
            "123",
            &vec![
                make_token(TokenType::Number, "123", (0, 3)),
                make_token(TokenType::EOF, "", (3, 4)),
            ]
        );
    }

    #[test]
    fn test_number_invalid() {
        assert_lex_error(
            ".1.23", 
            LexError::InvalidNumber { lexeme: ".1.23".to_string(), span: (0, 5) }
        );
        assert_lex_error(
            "1.23.", 
            LexError::InvalidNumber { lexeme: "1.23.".to_string(), span: (0, 5) }
        );
        assert_lex_error(
            ".", 
            LexError::InvalidNumber { lexeme: ".".to_string(), span: (0, 1) }
        );
        assert_lex_error(
            "..", 
            LexError::InvalidNumber { lexeme: "..".to_string(), span: (0, 2) }
        );
        assert_lex_error(
            ".123.", 
            LexError::InvalidNumber { lexeme: ".123.".to_string(), span: (0, 5) }
        );
    
    }

    #[test]
    fn test_negative_sign_is_operator() {
        assert_lex(
            "-1",
            &vec![
                make_token(TokenType::Minus, "-", (0, 1)),
                make_token(TokenType::Number, "1", (1, 2)),
                make_token(TokenType::EOF, "", (2, 3)),
            ]
        );
    }

    #[test]
    fn test_unexpected_char() {
        let bad_chars = vec!["@", "#", "!", ";"];
            for c in bad_chars {
                assert_lex_error(
                    c,
                    LexError::UnexpectedChar { char: c.to_string(), span: (0, 1) }
            );
        }
    }

}