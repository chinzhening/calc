#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    Ans,
    Const(f64),

    // Unary Operations
    Negate,
    
    // Binary Operations
    Add,
    Subtract,
    Times,
    Divide,
    Power,
    Root,

    // Functions
    Sin, Cos, Tan,
    ArcSin, ArcCos, ArcTan,
    Sqrt,

    Ln, Exp
}