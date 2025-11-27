use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {    
    LeftParen,
    RightParen,
    Comma,
    
    Minus,
    Plus,
    Slash,
    Star,

    Number,
    
    Sin, Cos, Tan,
    ArcSin, ArcCos, ArcTan,

    Ans,

    EOF,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub span: (usize, usize),
}
impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}({:?}, {:?})",
            self.token_type,
            self.lexeme,
            self.span,
        )
    }
}