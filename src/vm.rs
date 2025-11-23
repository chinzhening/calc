use std::fmt;

use crate::operation::{Operation};

macro_rules! binary_op {
    ($ops:ident, $op:tt) => {
        match ($ops.pop(), $ops.pop()) {
            (Some(x), Some(y)) => {
                $ops.push(y $op x);
            },
            _ => {},
        }
    };
}



#[derive(Debug)]
pub enum RuntimeError {
    DivisionByZero,
    InvalidOperation,
    Overflow,
    Underflow,
    FunctionDomainError,
}
impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct InterpretOutput {
    result: f64,
}
impl fmt::Display for InterpretOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Output: {}", self.result)
    }
}

pub fn interpret(operations: &Vec<Operation>) -> Result<InterpretOutput, RuntimeError> {
    let mut stack: Vec<f64> = Vec::new();
    operations.iter().for_each(|op|
        match op {
            Operation::Add => binary_op!(stack, +),
            Operation::Subtract => binary_op!(stack, -),
            Operation::Times => binary_op!(stack, *),
            Operation::Divide => binary_op!(stack, /),  // TODO: DivisionByZero, Inf, NaN
            Operation::Negate => match stack.pop() {
                Some(val) => stack.push(-val),
                _ => {},
            }
            Operation::Const(val) => {
                stack.push(val.clone());     // TODO: handle this better.
            },
        }
    );
    Ok(InterpretOutput { result: stack[0] }) // TODO: handle this better.
    
}