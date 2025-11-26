use std::collections::HashMap;
use std::fmt;

use crate::operation::Operation;
use crate::operation::Operation::*;

const EPS: f64 = 1e-10;
const EPS_INTERNAL: f64 = 1e-15;

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeError {
    MathError,
    DomainError,
    Underflow,
    NotImplemented,
}
impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterpretOutput {
    result: f64,
}
impl fmt::Display for InterpretOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Output: {}", self.result)
    }
}

pub struct VirtualMachine {
    pub use_radians: bool,
    table: HashMap<String, f64>,
}
impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            use_radians: true,
            table: HashMap::new(),
        }
    }

    pub fn interpret(
        &mut self,
        operations: &Vec<Operation>,
    ) -> Result<InterpretOutput, RuntimeError> {
        let stack = &mut Vec::new();

        for op in operations {
            match op {
                Add => interpret_add(stack)?,
                Subtract => interpret_subtract(stack)?,
                Times => interpret_times(stack)?,
                Divide => interpret_divide(stack)?,
                Negate => interpret_negate(stack)?,
                Sin | Cos | Tan => interpret_trig(stack, op, self.use_radians)?,
                ArcSin | ArcCos | ArcTan => interpret_inv_trig(stack, op, self.use_radians)?,
                Const(val) => stack.push(val.clone()), // TODO: handle this better.
                _ => {
                    return Err(RuntimeError::NotImplemented);
                }
            }
        }
        Ok(InterpretOutput { result: stack[0] }) // TODO: handle this better.
    }
}

fn interpret_add(stack: &mut Vec<f64>) -> Result<(), RuntimeError> {
    if let (Some(x), Some(y)) = (stack.pop(), stack.pop()) {
        stack.push(y + x);
        return Ok(());
    }

    Err(RuntimeError::Underflow)
}

fn interpret_subtract(stack: &mut Vec<f64>) -> Result<(), RuntimeError> {
    if let (Some(x), Some(y)) = (stack.pop(), stack.pop()) {
        stack.push(y - x);
        return Ok(());
    }

    Err(RuntimeError::Underflow)
}

fn interpret_times(stack: &mut Vec<f64>) -> Result<(), RuntimeError> {
    if let (Some(x), Some(y)) = (stack.pop(), stack.pop()) {
        stack.push(y * x);
        return Ok(());
    }

    Err(RuntimeError::Underflow)
}

fn interpret_divide(stack: &mut Vec<f64>) -> Result<(), RuntimeError> {
    if let (Some(x), Some(y)) = (stack.pop(), stack.pop()) {
        return if x == 0.0 {
            Err(RuntimeError::MathError)
        } else {
            stack.push(y / x);
            return Ok(());
        };
    }

    Err(RuntimeError::Underflow)
}

fn interpret_negate(stack: &mut Vec<f64>) -> Result<(), RuntimeError> {
    if let Some(val) = stack.pop() {
        stack.push(-val);
        return Ok(());
    }

    Err(RuntimeError::Underflow)
}

fn interpret_trig(
    stack: &mut Vec<f64>,
    op: &Operation,
    use_radians: bool,
) -> Result<(), RuntimeError> {
    if let Some(val) = stack.pop() {
        let operand = if use_radians { val } else { val.to_radians() };
        let result = match op {
            Sin => operand.sin(),
            Cos => operand.cos(),
            Tan => operand.tan(),
            _ => {
                return Err(RuntimeError::NotImplemented);
            }
        };

        stack.push(result);
        return Ok(());
    }

    Err(RuntimeError::Underflow)
}

fn interpret_inv_trig(
    stack: &mut Vec<f64>,
    op: &Operation,
    use_radians: bool,
) -> Result<(), RuntimeError> {
    if let Some(val) = stack.pop() {
        let result = match op {
            ArcSin => val.asin(),
            ArcCos => val.acos(),
            ArcTan => val.atan(),
            _ => {
                return Err(RuntimeError::NotImplemented);
            }
        };

        if result.is_nan() {
            return Err(RuntimeError::DomainError);
        }

        let result = if use_radians {
            result
        } else {
            result.to_degrees()
        };
        stack.push(result);

        return Ok(());
    }

    Err(RuntimeError::Underflow)
}

#[cfg(test)]
mod tests {
    use core::f64;
    use std::f64::consts::{FRAC_PI_4, FRAC_PI_2, PI};

    use super::*;

    fn eval(ops: Vec<Operation>) -> f64 {
        let mut vm = VirtualMachine::new();
        vm.interpret(&ops).unwrap().result
    }

    fn assert_approx_eq(a: f64, b: f64) {
        assert!(approx_eq(a, b, EPS_INTERNAL))
    }

    fn assert_runtime_error(ops: Vec<Operation>, expected_error: RuntimeError) {
        let mut vm = VirtualMachine::new();
        let result = vm.interpret(&ops);
        match result {
            Ok(_) => panic!("Expected runtime error {:?}, but got Ok", expected_error),
            Err(e) => assert_eq!(e, expected_error),
        }
    }

    fn approx_eq(a: f64, b: f64, eps: f64) -> bool {
        (a - b).abs() < eps
    }

    #[test]
    fn test_simple_const() {
        let ops = vec![Const(42.0)];
        assert_eq!(eval(ops), 42.0);
    }

    #[test]
    fn test_addition() {
        let ops = vec![Const(1.0), Const(2.0), Add];
        assert_eq!(eval(ops), 3.0);
    }

    #[test]
    fn test_subtraction() {
        let ops = vec![Const(10.0), Const(3.0), Subtract];
        assert_eq!(eval(ops), 7.0);
    }

    #[test]
    fn test_multiplication() {
        let ops = vec![Const(6.0), Const(7.0), Times];
        assert_eq!(eval(ops), 42.0);
    }

    #[test]
    fn test_division() {
        let ops = vec![Const(20.0), Const(4.0), Divide];
        assert_eq!(eval(ops), 5.0);
    }

    #[test]
    fn test_unary_negation() {
        let ops = vec![Const(5.0), Negate];
        assert_eq!(eval(ops), -5.0);
    }

    #[test]
    fn test_chained_expression() {
        // Equivalent to: 1 + 2 * 3  → RPN: 1 2 3 * +
        let ops = vec![Const(1.0), Const(2.0), Const(3.0), Times, Add];
        assert_eq!(eval(ops), 7.0);
    }

    #[test]
    fn test_division_by_zero() {
        let ops = vec![Const(5.0), Const(0.0), Divide];

        assert_runtime_error(ops, RuntimeError::MathError);
    }

    #[test]
    fn test_stack_underflow() {
        let ops = vec![Add];

        assert_runtime_error(ops, RuntimeError::Underflow);
    }

    #[test]
    fn test_overflow_behavior() {
        let ops = vec![Const(f64::MAX), Const(2.0), Times];

        let mut vm = VirtualMachine::new();
        let result = vm.interpret(&ops).unwrap().result;
        assert!(result.is_infinite());
    }

    #[test]
    fn test_sin() {
        let ops = vec![Const(2.0 * PI), Sin];
        assert_approx_eq(eval(ops), 0.0);
    }

    #[test]
    fn test_cos() {
        let ops = vec![Const(0.0), Cos];
        assert_approx_eq(eval(ops), 1.0);
    }

    #[test]
    fn test_tan() {
        let ops = vec![Const(FRAC_PI_2), Tan];
        assert!(eval(ops).abs() > 1.0 / EPS_INTERNAL);
    }

    #[test]
    fn test_arcsin() {
        let ops = vec![Const(1.0), ArcSin];
        assert_approx_eq(eval(ops), FRAC_PI_2);
    }

    #[test]
    fn test_arccos() {
        let ops = vec![Const(1.0), ArcCos];
        assert_approx_eq(eval(ops), 0.0);
    }

    #[test]
    fn test_arctan() {
        let ops = vec![Const(1.0), ArcTan];
        assert_approx_eq(eval(ops), FRAC_PI_4);
    }
}
