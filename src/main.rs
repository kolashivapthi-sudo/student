mod types;
mod error;
mod lexer;
mod filter;
mod translator;
mod flattener;
mod solver;
mod formatter;

use error::SolverError;
use formatter::{format_output, print_output};
use types::{Equation, Expr, Operator};

/// Simulates: "John wants to divide 25 apples into 0 baskets.
/// How many apples go into each basket?"
/// Expected: DivisionByZero error.
fn solve_division(total: f64, groups: f64) -> Result<f64, SolverError> {
    if groups == 0.0 {
        return Err(SolverError::DivisionByZero);
    }
    Ok(total / groups)
}

fn main() {
    // --- Demo 1: Division by zero error ---
    let total = 25.0;
    let baskets = 0.0;
    println!("Problem: John wants to divide {} apples into {} baskets.", total, baskets);
    println!("How many apples go into each basket?");
    println!();
    match solve_division(total, baskets) {
        Ok(answer) => println!("Answer: {} apples per basket.", answer),
        Err(e) => println!("Error: {}", e),
    }

    println!();
    println!("---");
    println!();

    // --- Demo 2: Full pipeline output with formatter ---
    // Simulates solving: "x = y + 3, y = 4"
    use std::collections::HashMap;
    let equations = vec![
        Equation::new(
            Expr::Variable("x".into()),
            Expr::BinaryOp {
                op: Operator::Add,
                left:  Box::new(Expr::Variable("y".into())),
                right: Box::new(Expr::Number(3.0)),
            },
        ),
        Equation::new(Expr::Variable("y".into()), Expr::Number(4.0)),
    ];

    match solver::solve(equations.clone()) {
        Ok(solution) => {
            let out = format_output(
                &equations,
                &solution.steps,
                &solution.values,
                &["x".to_string()],
            );
            print_output(&out);
        }
        Err(e) => println!("Error: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_division_by_zero_returns_error() {
        let result = solve_division(25.0, 0.0);
        assert_eq!(result, Err(SolverError::DivisionByZero));
    }

    #[test]
    fn test_division_by_zero_message() {
        let err = SolverError::DivisionByZero;
        assert_eq!(err.to_string(), "Division by zero is not possible.");
    }

    #[test]
    fn test_normal_division_works() {
        let result = solve_division(25.0, 5.0);
        assert_eq!(result, Ok(5.0));
    }
}
