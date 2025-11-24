#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    Const(f64),
    Negate,
    Add,
    Subtract,
    Times,
    Divide,
}