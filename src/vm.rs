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
    // InvalidOperation,
    Overflow,
    // Underflow,
    // FunctionDomainError,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation::Operation;

    fn eval(ops: Vec<Operation>) -> f64 {
        interpret(&ops).unwrap().result
    }

    #[test]
    fn test_simple_const() {
        let ops = vec![Operation::Const(42.0)];
        assert_eq!(eval(ops), 42.0);
    }

    #[test]
    fn test_addition() {
        let ops = vec![
            Operation::Const(1.0),
            Operation::Const(2.0),
            Operation::Add,
        ];
        assert_eq!(eval(ops), 3.0);
    }

    #[test]
    fn test_subtraction() {
        let ops = vec![
            Operation::Const(10.0),
            Operation::Const(3.0),
            Operation::Subtract,
        ];
        assert_eq!(eval(ops), 7.0);
    }

    #[test]
    fn test_multiplication() {
        let ops = vec![
            Operation::Const(6.0),
            Operation::Const(7.0),
            Operation::Times,
        ];
        assert_eq!(eval(ops), 42.0);
    }

    #[test]
    fn test_division() {
        let ops = vec![
            Operation::Const(20.0),
            Operation::Const(4.0),
            Operation::Divide,
        ];
        assert_eq!(eval(ops), 5.0);
    }

    #[test]
    fn test_unary_negation() {
        let ops = vec![
            Operation::Const(5.0),
            Operation::Negate,
        ];
        assert_eq!(eval(ops), -5.0);
    }

    #[test]
    fn test_chained_expression() {
        // Equivalent to: 1 + 2 * 3  → RPN: 1 2 3 * +
        let ops = vec![
            Operation::Const(1.0),
            Operation::Const(2.0),
            Operation::Const(3.0),
            Operation::Times,
            Operation::Add,
        ];
        assert_eq!(eval(ops), 7.0);
    }

    #[test]
    fn test_division_by_zero() {
        let ops = vec![
            Operation::Const(5.0),
            Operation::Const(0.0),
            Operation::Divide,
        ];

        let result = interpret(&ops);
        assert!(result.is_err()); // change to is_err() once div0 check exists
    }

    #[test]
    fn test_stack_underflow() {
        let ops = vec![
            Operation::Add, // not enough operands
        ];

        // Note: current code ignores the error, but intended behavior is tested.
        let result = interpret(&ops);
        assert!(result.is_err()); // change to is_err() after adding real error handling
    }

    #[test]
    fn test_overflow_behavior() {
        let ops = vec![
            Operation::Const(f64::MAX),
            Operation::Const(2.0),
            Operation::Times,
        ];

        let result = interpret(&ops).unwrap().result;
        assert!(result.is_infinite());
    }
}