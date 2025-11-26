#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    Const(f64),

    // Unary Operations
    Negate,
    
    // Binary Operations
    Add,
    Subtract,
    Times,
    Divide,

    // Functions
    Sin, Cos, Tan,
    ArcSin, ArcCos, ArcTan,
}