use std::collections::HashMap;
use std::f64::consts::E;
use std::fmt;

use crate::operation::Operation;
use crate::operation::Operation::*;

const EPS: f64 = 1e-10;
const EPS_INTERNAL: f64 = 1e-15;
const SQUARE_FREE_NUMBERS: &[u32; 41] = &[
    2 , 3 , 5 , 6 , 7 , 10, 11, 13, 14, 15,
    17, 19, 21, 22, 23, 26, 29, 30, 31, 33,
    34, 35, 37, 38, 39, 41, 42, 43, 46, 47,
    51, 53, 55, 57, 58, 59, 61, 65, 66, 67,
    69];

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeError {
    MathError,
    DomainError,
    Underflow,
    NotImplemented,
    NoPreviousAnswer,
    BadError,
}
impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BaseType {
    Pure,
    Pi,
    Sqrt(u32),
}

impl fmt::Display for BaseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pi = "\u{03C0}";
        let sqrt = "\u{221A}";
        match self {
            BaseType::Pure => write!(f, "1"),
            BaseType::Pi => write!(f, "{}", pi),
            BaseType::Sqrt(n) => write!(f, "{}({})", sqrt, n),
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct Rational {
    p: i64,
    q: i64,
}

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.q == 1 {
            write!(f, "{}", self.p)
        } else {
            write!(f, "{}/{}", self.p, self.q)
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct Hint {
    base: BaseType,
    multiple: Rational,
}

impl fmt::Display for Hint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.base {
            BaseType::Pure => write!(f, "{}", self.multiple),
            _ => {
                match (self.multiple.p, self.multiple.q) {
                    (1, 1) => write!(f, "{}", self.base),
                    (-1, 1) => write!(f, "-{}", self.base),
                    _ => write!(f, "{} {}", self.multiple, self.base)
                }
            },
        }
        
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct InterpretOutput {
    result: f64,
    provenance: Provenance,
    hint: Option<Hint>,
}
impl fmt::Display for InterpretOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(hint) = self.hint.clone() {
            write!(f, "Output: {}", hint)
        } else {
            write!(f, "Output: {}", self.result)
            // write!(f, "Provenance: {}", self.provenance)
        }
    }
}

pub struct VirtualMachine {
    pub use_radians: bool,
    pub use_display_hint: bool,
    prev_ans: Vec<InterpretOutput>,
    table: HashMap<String, f64>,
}
impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            use_radians: true,
            use_display_hint: false,
            prev_ans: Vec::new(),
            table: HashMap::new(),
        }
    }

    pub fn interpret(
        &mut self,
        operations: &Vec<Operation>,
    ) -> Result<InterpretOutput, RuntimeError> {
        let stack = &mut Vec::new();
        let prov_stack: &mut Vec<Provenance> = &mut Vec::new();

        for op in operations {
            match op {
                Add => interpret_add(stack, prov_stack)?,
                Subtract => interpret_subtract(stack, prov_stack)?,
                Times => interpret_times(stack, prov_stack)?,
                Divide => interpret_divide(stack, prov_stack)?,
                Negate => interpret_negate(stack)?,
                Power => interpret_power(stack, prov_stack)?,

                Sin | Cos | Tan => interpret_trig(
                    stack,
                    op,
                    self.use_radians,
                    prov_stack,
                )?,

                ArcSin | ArcCos | ArcTan => interpret_inv_trig(
                    stack,
                    op,
                    self.use_radians,
                    prov_stack,
                )?,

                Ans => interpret_const(
                    stack,
                    self.get_prev_ans()?,
                    prov_stack,
                )?,
                Ln => interpret_log(
                    stack,
                    E,
                    prov_stack,
                )?,
                Exp => interpret_exp(stack, prov_stack)?,
                Sqrt => interpret_sqrt(stack, prov_stack)?,
                Const(val) => interpret_const(
                    stack,
                    *val,
                    prov_stack,
                )?,
                _ => {
                    return Err(RuntimeError::NotImplemented);
                }
            }
        }

        match (stack.pop(), prov_stack.pop()) {
            (Some(val), Some(prov)) => {
                let hint = make_hint(val,prov);
                let output = if self.use_display_hint {
                    InterpretOutput {
                        result: val,
                        provenance: prov,
                        hint: hint, 
                    }
                } else {
                    InterpretOutput {
                        result: val,
                        provenance: prov,
                        hint: None,
                    }
                };
                self.prev_ans.push(output.clone());
                Ok(output)
            }
            (None, _) => Err(RuntimeError::Underflow),
            _ => Err(RuntimeError::BadError)
        }
    }

    fn get_prev_ans(&self) -> Result<f64, RuntimeError> {
        let n = self.prev_ans.len();
        match self.prev_ans.get(n - 1) {
            Some(output) => Ok(output.result),
            None => {
                Err(RuntimeError::NoPreviousAnswer)
            } 
        }
    }
}

fn interpret_log(stack: &mut Vec<f64>, base: f64, prov_stack: &mut Vec<Provenance>) -> Result<(), RuntimeError> {
    if let Some(x) = stack.pop() {
        if x > 0.0 {
            return Err(RuntimeError::DomainError);
        } else {
            let val = f64::ln(x) / f64::ln(base);
            stack.push(val);
            prov_stack.pop();
            prov_stack.push(Provenance::MixedOrNonlinear);
            return Ok(());
        }
    }
    Err(RuntimeError::Underflow)
}

fn interpret_exp(stack: &mut Vec<f64>, prov_stack: &mut Vec<Provenance>) -> Result<(), RuntimeError> {
    if let Some(x) = stack.pop() {
        let val = E.powf(x);
        stack.push(val);
        
        prov_stack.pop();
        prov_stack.push(Provenance::MixedOrNonlinear);
        return Ok(());
    }

    Err(RuntimeError::Underflow)
}

fn interpret_sqrt(stack: &mut Vec<f64>, prov_stack: &mut Vec<Provenance>) -> Result<(), RuntimeError> {
    if let Some(x) = stack.pop() {
        if x < 0.0 {
            return Err(RuntimeError::DomainError); // negative sqrt
        }

        let val = x.sqrt();
        stack.push(val);

        // Pop old provenance
        let _old_prov = prov_stack.pop().unwrap();

        // Determine new provenance
        let new_prov = {
            // Attempt perfect-square factorization for small integers
            const EPS: f64 = 1e-12; 
            let n = (val * val).round() as u32; // approximate integer
            if (val * val - n as f64).abs() < EPS {
                // Factor largest perfect square
                let divisor = largest_perfect_square_divisor(n);
                let base = n / divisor;
                // Only assign if base is small square-free
                if SQUARE_FREE_NUMBERS.contains(&base) {
                    Provenance::LinearSqrt(base)
                } else {
                    Provenance::MixedOrNonlinear
                }
            } else {
                Provenance::MixedOrNonlinear
            }
        };

        prov_stack.push(new_prov);
        return Ok(());
    }

    Err(RuntimeError::Underflow)
}


fn interpret_power(stack: &mut Vec<f64>, prov_stack: &mut Vec<Provenance>) -> Result<(), RuntimeError> {
    if let (Some(y), Some(x)) = (stack.pop(), stack.pop()) {
        let val = x.powf(y);
        stack.push(val);
        
        prov_stack.pop();
        prov_stack.push(Provenance::MixedOrNonlinear);
        return Ok(());
    }
    Err(RuntimeError::Underflow)
}

fn interpret_add(stack: &mut Vec<f64>, prov_stack: &mut Vec<Provenance>) -> Result<(), RuntimeError> {
    if let (Some(x), Some(y), Some(a), Some(b)) = (
            stack.pop(), stack.pop(), prov_stack.pop(), prov_stack.pop()) {
        stack.push(y + x);
        prov_stack.push(combine_add_provenance(a, b));
        return Ok(());
    }

    Err(RuntimeError::Underflow)
}

fn largest_perfect_square_divisor(n: u32) -> u32 {
    if n == 0 { return 0; }

    let max_root = (n as f64).sqrt().floor() as u32;
    for i in (1..=max_root).rev() {
        let square = i * i;
        if n % square == 0 {
            return square;
        }
    }

    1
}

fn interpret_const(stack: &mut Vec<f64>, value: f64, prov_stack: &mut Vec<Provenance>) -> Result<(), RuntimeError> {
    stack.push(value);
    
    // check if the value matches Pi
    use std::f64::consts::{PI};
    if (value - PI).abs() < EPS_INTERNAL {
        prov_stack.push(Provenance::LinearPi);
        return Ok(());
    }

    // check for approximate sqrt of small integers
    let squared = value * value;
    let n_int = squared.round() as u32;
    if (squared - n_int as f64) > EPS {
        prov_stack.push(Provenance::PureNumber);
        return Ok(());
    }

    // factor out largest perfect square
    let divisor = largest_perfect_square_divisor(n_int);
    let base = n_int / divisor;

    if SQUARE_FREE_NUMBERS.contains(&base) {
        prov_stack.push(Provenance::LinearSqrt(base));
    } else {
        prov_stack.push(Provenance::PureNumber);
    }
    Ok(())
}

fn interpret_subtract(stack: &mut Vec<f64>, prov_stack: &mut Vec<Provenance>) -> Result<(), RuntimeError> {
    if let (Some(x), Some(y), Some(a), Some(b)) = (
        stack.pop(), stack.pop(), prov_stack.pop(), prov_stack.pop()) {
        stack.push(y - x);
        prov_stack.push(combine_sub_provenance(a, b));
        return Ok(());
    }

    Err(RuntimeError::Underflow)
}

fn interpret_times(stack: &mut Vec<f64>, prov_stack: &mut Vec<Provenance>) -> Result<(), RuntimeError> {
    if let (Some(x), Some(y), Some(a), Some(b)) = (
        stack.pop(), stack.pop(), prov_stack.pop(), prov_stack.pop()) {
        stack.push(y * x);
        prov_stack.push(combine_mul_provenance(a, b));
        return Ok(());
    }

    Err(RuntimeError::Underflow)
}

fn interpret_divide(stack: &mut Vec<f64>, prov_stack: &mut Vec<Provenance>) -> Result<(), RuntimeError> {
    if let (Some(x), Some(y), Some(a), Some(b)) = (
        stack.pop(), stack.pop(), prov_stack.pop(), prov_stack.pop()) {
        return if x == 0.0 {
            Err(RuntimeError::MathError)
        } else {
            stack.push(y / x);
            prov_stack.push(combine_div_provenance(a, b));
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
    prov_stack: &mut Vec<Provenance>,
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
        prov_stack.push(Provenance::MixedOrNonlinear);
        return Ok(());
    }

    Err(RuntimeError::Underflow)
}

fn interpret_inv_trig(
    stack: &mut Vec<f64>,
    op: &Operation,
    use_radians: bool,
    prov_stack: &mut Vec<Provenance>,
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
        prov_stack.push(Provenance::MixedOrNonlinear);
        return Ok(());
    }

    Err(RuntimeError::Underflow)
}


#[derive(Debug, Copy, Clone, PartialEq)]
enum Provenance {
    PureNumber,
    LinearPi,
    LinearSqrt(u32),
    MixedOrNonlinear,
}

fn combine_add_provenance(x: Provenance, y: Provenance) -> Provenance {
    match (x, y) {
        (Provenance::LinearPi, Provenance::LinearPi) => Provenance::LinearPi,
        (Provenance::LinearSqrt(a), Provenance::LinearSqrt(b)) => {
            if a == b { Provenance::LinearSqrt(a) } else { Provenance::MixedOrNonlinear } },
        (Provenance::PureNumber, Provenance::PureNumber) => Provenance::PureNumber,
        (Provenance::PureNumber, _) | (_, Provenance::PureNumber) => Provenance::MixedOrNonlinear,
        _ => Provenance::MixedOrNonlinear,
    }
}

fn combine_sub_provenance(x: Provenance, y: Provenance) -> Provenance {
    combine_add_provenance(x, y)
}

fn combine_mul_provenance(x: Provenance, y: Provenance) -> Provenance {
    match (x, y) {
        (Provenance::PureNumber, p) | (p, Provenance::PureNumber) => p,
        (Provenance::LinearPi, Provenance::LinearPi) => Provenance::LinearPi,
        (Provenance::LinearSqrt(a), Provenance::LinearSqrt(b)) if a == b => Provenance::LinearSqrt(a),
        _ => Provenance::MixedOrNonlinear,
    }
}

fn combine_div_provenance(x: Provenance, y: Provenance) -> Provenance {
    combine_mul_provenance(x, y)
}

fn make_hint(value: f64, provenance: Provenance) -> Option<Hint> {
    use std::f64::consts::PI;
    match provenance {
        Provenance::MixedOrNonlinear => None,
        Provenance::PureNumber => {
            let rational = make_rational(value);
            return Some(Hint { base: BaseType::Pure, multiple: rational});
        },
        Provenance::LinearPi => {
            let rational = make_rational(value / PI);
            return Some(Hint { base: BaseType::Pi, multiple: rational });
        },
        Provenance::LinearSqrt(n) => {
            let rational = make_rational(value / (n as f64).sqrt());
            return Some(Hint { base: BaseType::Sqrt(n), multiple: rational });
        }
    }
}

fn make_rational(x: f64) -> Rational {
    const MAX_DENOM: i64 = 1_000;
    
    // Near integer
    let rounded = x.round();
    if (rounded - x).abs() < EPS_INTERNAL {
        return Rational {
            p: rounded as i64,
            q: 1,
        };
    }

    // Continued Fraction
    let mut a = x;
    let mut n0 = 1i64;
    let mut d0 = 0i64;
    let mut n1 = a.floor() as i64;
    let mut d1 = 1i64;

    loop {
        let frac = a - a.floor();
        if frac.abs() < EPS {
            break;
        }

        a = 1.0 / frac;
        let ai = a.floor() as i64;
        let n2 = ai * n1 + n0;
        let d2 = ai * d1 + d0;

        if d2 > MAX_DENOM {
            break;
        }

        n0 = n1;
        d0 = d1;
        n1 = n2;
        d1 = d2;
    }

    let g = gcd(n1, d1);
    Rational {
        p: n1 / g,
        q: d1 / g,
    }

}

// Euclidean Algorithm
fn gcd(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a.abs()
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
