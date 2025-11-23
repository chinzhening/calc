#[derive(Debug, Clone)]
pub enum Operation {
    Const(f64),
    Negate,
    Add,
    Subtract,
    Times,
    Divide,
}