use std::fmt;

use crate::token::{Token, TokenType};
use crate::operation::{Operation};

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


#[derive(Debug, Clone)]
pub enum ParseError {
    ExpectExpression { token: Token },
    ExpectEndOfExpression { token: Token },
    ExpectRightParenAfterExpression { token: Token },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::ExpectExpression { token } => {
                write!(f, "Expected an expression at {}", token.span.0)
            }
            ParseError::ExpectEndOfExpression { token } => {
                write!(f, "Expected the end of expression at {}", token.span.0)
            }
            ParseError::ExpectRightParenAfterExpression { token } => {
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
    pub fn new(tokens: &Vec<Token>) -> Self {
        Self { tokens: tokens.clone(), operations: Vec::new(), curr: 0, prev: 0 }
    }
    fn get_parse_rule(token_type: &TokenType) -> ParseRule {
        match token_type  {
            TokenType::LeftParen => ParseRule {
                prefix: Some(|parser| parser.grouping() ),
                infix: None,
                precedence: Precedence::None 
            },
            TokenType::Minus => ParseRule { 
                prefix: Some(|parser| parser.unary() ),
                infix: Some(|parser| parser.binary() ),
                precedence: Precedence::Term
            }, 
            TokenType::Plus => ParseRule {
                prefix: None,
                infix: Some(|parser| parser.binary() ),
                precedence: Precedence::Term
            }, 
            TokenType::Slash => ParseRule {
                prefix: None,
                infix: Some(|parser| parser.binary() ),
                precedence: Precedence::Factor
            }, 
            TokenType::Star => ParseRule {
                prefix: None,
                infix: Some(|parser| parser.binary() ),
                precedence: Precedence::Factor
            }, 
            TokenType::Number => ParseRule {
                prefix: Some(|parser| parser.number() ),
                infix: None,
                precedence: Precedence::None
            },
            _ => ParseRule {
                prefix: None,
                infix: None,
                precedence: Precedence::None
            },
        }
    }

    pub fn parse(&mut self) -> Result<&Vec<Operation>, ParseError> {
        self.expression()?;
        self.consume(TokenType::EOF)?;
        Ok(&self.operations)
    }

    fn expression(&mut self) -> Result<(), ParseError> {
        self.parse_precedence(Precedence::Term)?;
        Ok(())
    }

    fn grouping(&mut self) -> Result<(), ParseError> {
        self.expression()?;
        self.consume(TokenType::RightParen)?;
        Ok(())
    }

    fn unary(&mut self) -> Result<(), ParseError> {
        let prev_token_type = self.prev().token_type.clone();
        self.parse_precedence(Precedence::Unary)?;
        
        match prev_token_type {
            TokenType::Minus => self.operations.push(Operation::Negate),
            _ => {},
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
            _ => {},
        }
        Ok(())
    }

    fn number(&mut self) -> Result<(), ParseError> {
        let val = self.prev().lexeme.parse::<f64>().unwrap();
        self.operations.push(Operation::Const(val));
        Ok(())
    }

    fn consume(&mut self, token_type: TokenType) -> Result<(), ParseError> {
        if self.curr().token_type == token_type {
            self.advance();
            return Ok(());
        }
        match self.curr().token_type {
            TokenType::RightParen => {
                Err(ParseError::ExpectRightParenAfterExpression { token: self.curr().clone() })
            },
            TokenType::EOF => {
                Err(ParseError::ExpectEndOfExpression { token: self.curr().clone() })
            }
            _ => Ok(()),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> Result<(), ParseError> {
        self.advance();
        let prev_token_type = self.prev().token_type.clone();
        let prefix_rule: Option<fn(&mut Parser) -> Result<(), ParseError>> = Self::get_parse_rule(&prev_token_type).prefix;
        match prefix_rule {
            None => {
                return Err(ParseError::ExpectExpression { token: self.prev().clone() });
            },
            Some(prefix_rule) => {
                prefix_rule(self)?;
            },
        }

        while precedence <= Self::get_parse_rule(&self.curr().token_type.clone()).precedence {
            self.advance();
            let prev_token_type = self.prev().token_type.clone();
            let infix_rule = Self::get_parse_rule(&prev_token_type).infix;
            match infix_rule {
                None => {},      // unreachable?
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

    fn curr(&self) -> &Token {
        &self.tokens[self.curr]
    }

    fn prev(&self) -> &Token {
        &self.tokens[self.prev]
    }

}

