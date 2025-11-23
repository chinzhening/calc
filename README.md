# Calc
Calc is a command-line calculator build in Rust. It supports basic 
arithmetic operations, parentheses for grouping, and operator precedence.

The calculator uses a Pratt parser to parse mathematical expressions into
a sequence of operations, which are then evaluated to produce a result.

## Bugs
- Division by zero is handled by Rust `f64` type, but is displayed as 0 or -0 instead of 'Inf' or '-Inf'.

## Feature List
- [ ] Unit Testing
- [ ] Functions, e.g. sin, cos, tan, log, exp
- [ ] Previous Result refernced by `Ans`
- [ ] Pretty IO
- [ ] Scientific Notation Support

## Extensions (Graphing Calculator)
- [ ] Solving Equations
- [ ] Graphing Support