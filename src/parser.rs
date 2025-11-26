use std::fmt;

use crate::operation::Operation;
use crate::token::*;


pub fn parse(tokens: Vec<Token>) -> Result<Vec<Operation>, ParseError> {
    let mut parser = Parser::new();
    parser.parse(&tokens).cloned()
}



#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    None,
    Term,
    Factor, 
    Unary,
    Call,
    Primary,
}
impl Precedence {
    fn next(self) -> Self {
        use Precedence::*;
        match self {
            None => Term,
            Term => Factor,
            Factor => Unary,
            Unary => Call,
            Call => Primary,
            Primary => Primary,
        }
    }
}

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    operations: Vec<Operation>,
    curr: usize,
    prev: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    ExpectExpression { token: Token },
    ExpectEndOfExpression,
    ExpectRightParenAfterExpression { token: Token },
}

use ParseError::*;

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExpectExpression { token } => {
                write!(f, "Expected an expression at {}", token.span.0)
            }
            ExpectEndOfExpression => {
                write!(f, "Expected the end of expression")
            }
            ExpectRightParenAfterExpression { token } => {
                write!(f, "Expected ')' after expression at {}", token.span.0)
            }
        }
    }
}

struct ParseRule {
    prefix: Option<fn(&mut Parser) -> Result<(), ParseError>>,
    infix: Option<fn(&mut Parser) -> Result<(), ParseError>>,
    precedence: Precedence,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            operations: Vec::new(),
            curr: 0,
            prev: 0,
        }
    }
    fn get_parse_rule(token_type: &TokenType) -> ParseRule {
        use TokenType::*;
        match token_type {
            LeftParen => ParseRule {
                prefix: Some(|parser| parser.grouping()),
                infix: None,
                precedence: Precedence::Call,
            },
            Minus => ParseRule {
                prefix: Some(|parser| parser.unary()),
                infix: Some(|parser| parser.binary()),
                precedence: Precedence::Term,
            },
            Plus => ParseRule {
                prefix: None,
                infix: Some(|parser| parser.binary()),
                precedence: Precedence::Term,
            },
            Slash => ParseRule {
                prefix: None,
                infix: Some(|parser| parser.binary()),
                precedence: Precedence::Factor,
            },
            Star => ParseRule {
                prefix: None,
                infix: Some(|parser| parser.binary()),
                precedence: Precedence::Factor,
            },
            Number => ParseRule {
                prefix: Some(|parser| parser.number()),
                infix: None,
                precedence: Precedence::None,
            },
            Sin | Cos | Tan | ArcSin | ArcCos | ArcTan => ParseRule {
                prefix: Some(|parser| parser.unary()),
                infix: None,
                precedence: Precedence::Term,
            },
            _ => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None,
            },
        }
    }

    pub fn parse(&mut self, tokens: &Vec<Token>) -> Result<&Vec<Operation>, ParseError> {
        self.tokens = tokens.clone();
        self.expression()?;
        self.consume(TokenType::EOF, 
            |_| ExpectEndOfExpression
        )?;
        Ok(&self.operations)
    }

    fn expression(&mut self) -> Result<(), ParseError> {
        self.parse_precedence(Precedence::Term)?;
        Ok(())
    }

    fn grouping(&mut self) -> Result<(), ParseError> {
        self.expression()?;
        self.consume(TokenType::RightParen, |s| {
                ExpectRightParenAfterExpression { token: s.curr().clone() }
        })?;
        Ok(())
    }

    fn unary(&mut self) -> Result<(), ParseError> {
        let prev_token_type = self.prev().token_type.clone();
        self.parse_precedence(Precedence::Unary)?;

        match prev_token_type {
            TokenType::Minus => self.operations.push(Operation::Negate),
            TokenType::Sin => self.operations.push(Operation::Sin),
            TokenType::Cos => self.operations.push(Operation::Cos),
            TokenType::Tan => self.operations.push(Operation::Tan),
            TokenType::ArcSin => self.operations.push(Operation::ArcSin),
            TokenType::ArcCos => self.operations.push(Operation::ArcCos),
            TokenType::ArcTan => self.operations.push(Operation::ArcTan),
            _ => {}
        }
        Ok(())
    }

    fn binary(&mut self) -> Result<(), ParseError> {
        let operator_type = self.prev().token_type.clone();
        let parse_rule = Self::get_parse_rule(&operator_type);
        self.parse_precedence(parse_rule.precedence.next())?;

        match operator_type {
            TokenType::Plus => self.operations.push(Operation::Add),
            TokenType::Minus => self.operations.push(Operation::Subtract),
            TokenType::Star => self.operations.push(Operation::Times),
            TokenType::Slash => self.operations.push(Operation::Divide),
            _ => {}
        }
        Ok(())
    }

    fn number(&mut self) -> Result<(), ParseError> {
        let val = self.prev().lexeme.parse::<f64>().unwrap();
        self.operations.push(Operation::Const(val));
        Ok(())
    }

    fn consume<F>(&mut self, token_type: TokenType, err: F) -> Result<(), ParseError>
    where
        F: FnOnce(&mut Self) -> ParseError,
    {
        if !self.is_at_end() && self.check(token_type) {
            self.advance();
            Ok(())
        } else {
            Err(err(self))
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> Result<(), ParseError> {
        self.advance();
        let prev_token_type = self.prev().token_type.clone();
        let prefix_rule: Option<fn(&mut Parser) -> Result<(), ParseError>> =
            Self::get_parse_rule(&prev_token_type).prefix;
        match prefix_rule {
            None => {
                return Err(ParseError::ExpectExpression {
                    token: self.prev().clone(),
                });
            }
            Some(prefix_rule) => {
                prefix_rule(self)?;
            }
        }

        while !self.is_at_end() {
            // TODO: handle this more elegantly.
            if precedence > Self::get_parse_rule(&self.curr().token_type.clone()).precedence {
                break;
            }
            self.advance();
            let prev_token_type = self.prev().token_type.clone();
            let infix_rule = Self::get_parse_rule(&prev_token_type).infix;
            match infix_rule {
                None => {}
                Some(infix_rule) => {
                    infix_rule(self)?;
                }
            }
        }

        Ok(())
    }

    fn advance(&mut self) {
        self.prev = self.curr;
        self.curr += 1;
    }

    fn check(&self, token_type: TokenType) -> bool {
        self.curr().token_type == token_type
    }

    /* Might cause panic */
    fn curr(&self) -> &Token {
        &self.tokens[self.curr]
    }

    fn prev(&self) -> &Token {
        &self.tokens[self.prev]
    }

    fn is_at_end(&self) -> bool {
        self.curr >= self.tokens.len()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation::Operation;
    use crate::token::{Token, TokenType};

    fn make_token<S: Into<String>>(
        token_type: TokenType,
        lexeme: S,
        span: (usize, usize),
    ) -> Token {
        Token {
            token_type,
            lexeme: lexeme.into(),
            span,
        }
    }

    fn assert_parse(tokens: Vec<Token>, expected_ops: &[Operation]) {
        let mut parser = Parser::new();
        let ops = parser.parse(&tokens).expect("Parser failed");
        assert_eq!(ops.as_slice(), expected_ops);
    }

    fn assert_parse_error(tokens: Vec<Token>, expected_error: ParseError) {
        let mut parser = Parser::new();
        let result = parser.parse(&tokens);
        match result {
            Ok(_) => panic!("Expected parser error {:?}, but got Ok", expected_error),
            Err(e) => assert_eq!(e, expected_error),
        }
    }

    #[test]
    fn test_single_number() {
        assert_parse(
            vec![
                make_token(TokenType::Number, "42", (0, 2)),
                make_token(TokenType::EOF, "", (2, 2)),
            ],
            &[Operation::Const(42.0)],
        );
    }

    #[test]
    fn test_unary_minus() {
        assert_parse(
            vec![
                make_token(TokenType::Minus, "-", (0, 1)),
                make_token(TokenType::Number, "7", (1, 2)),
                make_token(TokenType::EOF, "", (2, 2)),
            ],
            &[Operation::Const(7.0), Operation::Negate],
        );
    }

    #[test]
    fn test_simple_addition() {
        assert_parse(
            vec![
                make_token(TokenType::Number, "1", (0, 1)),
                make_token(TokenType::Plus, "+", (1, 2)),
                make_token(TokenType::Number, "2", (2, 3)),
                make_token(TokenType::EOF, "", (3, 3)),
            ],
            &[Operation::Const(1.0), Operation::Const(2.0), Operation::Add],
        );
    }

    #[test]
    fn test_missing_expression() {
        // EOF where an expression is expected. the parse() method immediately
        // searches for an expression() by default. This could change in the future.
        assert_parse_error(
            vec![make_token(TokenType::EOF, "", (0, 1))],
            ParseError::ExpectExpression {
                token: make_token(TokenType::EOF, "", (0, 1)),
            },
        );

        // Binary Operation followed by EOF where parser expects a right operand.
        assert_parse_error(
            vec![
                make_token(TokenType::Number, "1", (0, 1)),
                make_token(TokenType::Plus, "+", (1, 2)),
                make_token(TokenType::EOF, "", (2, 3)),
            ],
            ParseError::ExpectExpression {
                token: make_token(TokenType::EOF, "", (2, 3)),
            },
        );
    }

    #[test]
    fn test_missing_right_paren() {
        // Open parenthesis without a matching right parenthesis
        assert_parse_error(
            vec![
                make_token(TokenType::LeftParen, "(", (0, 1)),
                make_token(TokenType::Number, "1", (1, 2)),
                make_token(TokenType::EOF, "", (2, 3)),
            ],
            ParseError::ExpectRightParenAfterExpression {
                token: make_token(TokenType::EOF, "", (2, 3)),
            },
        );
    }

    #[test]
    fn test_unexpected_end_of_expression() {
        // EOF token missing
        assert_parse_error(
            vec![
                make_token(TokenType::Number, "1", (0, 1)),
                make_token(TokenType::Plus, "+", (1, 2)),
                make_token(TokenType::Number, "1", (2, 3)),
            ],
            ExpectEndOfExpression,
        );
    }
}
