use std::collections::HashMap;
use crate::error::SolverError;
use crate::types::{Equation, Expr, Operator};

// ---------------------------------------------------------------------------
// Scope guards — run BEFORE solving
// ---------------------------------------------------------------------------

/// Keywords that signal out-of-scope domains (trigonometry, calculus).
const UNSUPPORTED_DOMAIN_KEYWORDS: &[&str] = &[
    "sin", "cos", "tan", "cot", "sec", "csc",
    "log", "ln", "exp",
    "integral", "derivative", "limit", "differential",
    "vector", "matrix", "determinant",
];

/// Check every variable name in the equation list for out-of-scope keywords.
/// Translator should catch these, but we add a defensive layer here.
fn check_domain(equations: &[Equation]) -> Result<(), SolverError> {
    for eq in equations {
        check_expr_domain(&eq.lhs)?;
        check_expr_domain(&eq.rhs)?;
    }
    Ok(())
}

fn check_expr_domain(expr: &Expr) -> Result<(), SolverError> {
    match expr {
        Expr::Variable(name) => {
            let lower = name.to_lowercase();
            // Use whole-word matching: split on underscores and check each part.
            // This prevents "second" matching "sec", "cosine" matching "cos", etc.
            let parts: Vec<&str> = lower.split('_').collect();
            for kw in UNSUPPORTED_DOMAIN_KEYWORDS {
                // Match only if the keyword equals the whole variable name
                // OR equals one of its underscore-separated parts.
                if lower == *kw || parts.iter().any(|p| p == kw) {
                    return Err(SolverError::UnsupportedDomain(name.clone()));
                }
            }
            Ok(())
        }
        Expr::Number(_) => Ok(()),
        Expr::BinaryOp { left, right, .. } => {
            check_expr_domain(left)?;
            check_expr_domain(right)
        }
    }
}

/// Detect degree > 1: a variable multiplied by itself in the same BinaryOp.
/// E.g. `x * x` or `x * (x + 1)` (the latter signals degree 2 via nested mul).
/// We use a simple heuristic: if both operands of a Mul are non-constant
/// (contain variables), we flag it as potentially quadratic (degree 2).
fn check_degree(equations: &[Equation]) -> Result<(), SolverError> {
    for eq in equations {
        check_expr_degree(&eq.lhs, 1)?;
        check_expr_degree(&eq.rhs, 1)?;
    }
    Ok(())
}

/// Walk an Expr and track the "degree" implied by nested multiplications
/// of non-constant sub-expressions.
fn check_expr_degree(expr: &Expr, current_degree: u32) -> Result<(), SolverError> {
    match expr {
        Expr::Number(_) | Expr::Variable(_) => Ok(()),
        Expr::BinaryOp { op: Operator::Mul, left, right } => {
            let left_has_var  = contains_variable(left);
            let right_has_var = contains_variable(right);
            let new_degree = if left_has_var && right_has_var {
                current_degree + 1
            } else {
                current_degree
            };
            if new_degree > 1 {
                return Err(SolverError::UnsupportedDegree(new_degree));
            }
            check_expr_degree(left,  new_degree)?;
            check_expr_degree(right, new_degree)
        }
        Expr::BinaryOp { left, right, .. } => {
            check_expr_degree(left,  current_degree)?;
            check_expr_degree(right, current_degree)
        }
    }
}

/// Returns true if `expr` contains at least one `Expr::Variable`.
fn contains_variable(expr: &Expr) -> bool {
    match expr {
        Expr::Variable(_) => true,
        Expr::Number(_)   => false,
        Expr::BinaryOp { left, right, .. } => {
            contains_variable(left) || contains_variable(right)
        }
    }
}

// ---------------------------------------------------------------------------
// Variable counting
// ---------------------------------------------------------------------------

/// Count how many *distinct unknown* variables appear in `expr`,
/// given a map of already-solved knowns.
fn count_unknowns(expr: &Expr, known: &HashMap<String, f64>) -> usize {
    collect_unknowns(expr, known).len()
}

/// Collect the set of unknown variable names in `expr`.
fn collect_unknowns(expr: &Expr, known: &HashMap<String, f64>) -> Vec<String> {
    let mut vars = Vec::new();
    collect_unknowns_inner(expr, known, &mut vars);
    vars.sort();
    vars.dedup();
    vars
}

fn collect_unknowns_inner(expr: &Expr, known: &HashMap<String, f64>, out: &mut Vec<String>) {
    match expr {
        Expr::Number(_) => {}
        Expr::Variable(name) => {
            if !known.contains_key(name.as_str()) {
                out.push(name.clone());
            }
        }
        Expr::BinaryOp { left, right, .. } => {
            collect_unknowns_inner(left,  known, out);
            collect_unknowns_inner(right, known, out);
        }
    }
}

// ---------------------------------------------------------------------------
// Substitution
// ---------------------------------------------------------------------------

/// Replace every `Expr::Variable(name)` that exists in `known` with its
/// `Expr::Number(value)`, recursively.
fn substitute(expr: Expr, known: &HashMap<String, f64>) -> Expr {
    match expr {
        Expr::Variable(ref name) => {
            if let Some(&val) = known.get(name.as_str()) {
                Expr::Number(val)
            } else {
                expr
            }
        }
        Expr::Number(_) => expr,
        Expr::BinaryOp { op, left, right } => Expr::BinaryOp {
            op,
            left:  Box::new(substitute(*left,  known)),
            right: Box::new(substitute(*right, known)),
        },
    }
}

/// Substitute knowns into both sides of every equation in the list.
fn substitute_all(equations: &mut Vec<Equation>, known: &HashMap<String, f64>) {
    for eq in equations.iter_mut() {
        eq.lhs = substitute(eq.lhs.clone(), known);
        eq.rhs = substitute(eq.rhs.clone(), known);
    }
}

// ---------------------------------------------------------------------------
// Evaluate a fully-known expression
// ---------------------------------------------------------------------------

/// Evaluate an `Expr` that should contain no unknowns.
/// Returns `Err(SolverError::DivisionByZero)` if a division by zero is found.
fn evaluate(expr: &Expr) -> Result<f64, SolverError> {
    match expr {
        Expr::Number(n) => Ok(*n),
        Expr::Variable(name) => Err(SolverError::ParseError(
            format!("Variable '{}' is still unknown during evaluation.", name),
        )),
        Expr::BinaryOp { op, left, right } => {
            let l = evaluate(left)?;
            let r = evaluate(right)?;
            match op {
                Operator::Add => Ok(l + r),
                Operator::Sub => Ok(l - r),
                Operator::Mul => Ok(l * r),
                Operator::Div => {
                    if r == 0.0 {
                        Err(SolverError::DivisionByZero)
                    } else {
                        Ok(l / r)
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Isolate: algebraic rearrangement
// ---------------------------------------------------------------------------

/// Given an equation where exactly one unknown variable `var` exists,
/// rearrange it to produce the numeric value of `var`.
///
/// Handles these flat shapes (post-flattener, rhs is at most one BinaryOp):
///   var = <known_expr>               → evaluate rhs
///   <known_expr> = var               → evaluate lhs
///   var OP known = known             → invert OP
///   known OP var = known             → invert OP (commuted)
fn isolate(eq: &Equation, var: &str, known: &HashMap<String, f64>) -> Result<f64, SolverError> {
    // Substitute all known values first to reduce the equation
    let lhs = substitute(eq.lhs.clone(), known);
    let rhs = substitute(eq.rhs.clone(), known);

    // Case 1: var = <fully known rhs>
    if matches!(&lhs, Expr::Variable(n) if n == var) {
        if count_unknowns(&rhs, known) == 0 {
            return evaluate(&rhs);
        }
    }

    // Case 2: <fully known lhs> = var
    if matches!(&rhs, Expr::Variable(n) if n == var) {
        if count_unknowns(&lhs, known) == 0 {
            return evaluate(&lhs);
        }
    }

    // Case 3: BinaryOp on rhs containing the unknown
    if let Expr::BinaryOp { op, left, right } = &rhs {
        let left_is_var  = matches!(left.as_ref(),  Expr::Variable(n) if n == var);
        let right_is_var = matches!(right.as_ref(), Expr::Variable(n) if n == var);
        let lhs_val = evaluate(&lhs)?;

        if left_is_var && count_unknowns(right, known) == 0 {
            let r = evaluate(right)?;
            return invert_op(op, lhs_val, r, true);
        }
        if right_is_var && count_unknowns(left, known) == 0 {
            let l = evaluate(left)?;
            return invert_op(op, lhs_val, l, false);
        }
    }

    // Case 4: BinaryOp on lhs containing the unknown
    if let Expr::BinaryOp { op, left, right } = &lhs {
        let left_is_var  = matches!(left.as_ref(),  Expr::Variable(n) if n == var);
        let right_is_var = matches!(right.as_ref(), Expr::Variable(n) if n == var);
        let rhs_val = evaluate(&rhs)?;

        if left_is_var && count_unknowns(right, known) == 0 {
            let r = evaluate(right)?;
            return invert_op(op, rhs_val, r, true);
        }
        if right_is_var && count_unknowns(left, known) == 0 {
            let l = evaluate(left)?;
            return invert_op(op, rhs_val, l, false);
        }
    }

    Err(SolverError::InsufficientInformation)
}

/// Invert a binary operation to solve for the unknown operand.
///
/// If `unknown_is_left`:  unknown OP known = result  →  unknown = result INV_OP known
/// If `!unknown_is_left`: known OP unknown = result  →  unknown = result INV_OP known
fn invert_op(op: &Operator, result: f64, known: f64, unknown_is_left: bool) -> Result<f64, SolverError> {
    match (op, unknown_is_left) {
        // x + k = r  →  x = r - k
        (Operator::Add, true)  => Ok(result - known),
        // k + x = r  →  x = r - k
        (Operator::Add, false) => Ok(result - known),
        // x - k = r  →  x = r + k
        (Operator::Sub, true)  => Ok(result + known),
        // k - x = r  →  x = k - r
        (Operator::Sub, false) => Ok(known - result),
        // x * k = r  →  x = r / k
        (Operator::Mul, true)  => {
            if known == 0.0 { Err(SolverError::DivisionByZero) } else { Ok(result / known) }
        }
        // k * x = r  →  x = r / k
        (Operator::Mul, false) => {
            if known == 0.0 { Err(SolverError::DivisionByZero) } else { Ok(result / known) }
        }
        // x / k = r  →  x = r * k
        (Operator::Div, true)  => Ok(result * known),
        // k / x = r  →  x = k / r
        (Operator::Div, false) => {
            if result == 0.0 { Err(SolverError::DivisionByZero) } else { Ok(known / result) }
        }
    }
}

// ---------------------------------------------------------------------------
// Public API — constraint propagation solver
// ---------------------------------------------------------------------------

/// Result of a successful solve: maps variable names to their values,
/// plus a trace of the steps taken (for output display).
#[derive(Debug, Clone, PartialEq)]
pub struct Solution {
    pub values: HashMap<String, f64>,
    pub steps:  Vec<String>,       // human-readable trace
}

/// Solve a flat list of equations using constraint propagation.
///
/// Algorithm (5-step loop):
///   1. Find an equation with exactly one unknown variable.
///   2. Isolate that variable algebraically.
///   3. Compute its value.
///   4. Record it in `known`; substitute into all remaining equations.
///   5. Remove the solved equation; recurse.
///
/// Returns `SolverError::InsufficientInformation` if a full pass makes no
/// progress and unknowns remain.
pub fn solve(equations: Vec<Equation>) -> Result<Solution, SolverError> {
    // --- Pre-flight scope checks ---
    check_domain(&equations)?;
    check_degree(&equations)?;

    let mut known:  HashMap<String, f64> = HashMap::new();
    let mut steps:  Vec<String>          = Vec::new();
    let mut remaining: Vec<Equation>     = equations;

    loop {
        // Remove equations that are now fully known (both sides evaluate)
        remaining.retain(|eq| {
            count_unknowns(&eq.lhs, &known) > 0 || count_unknowns(&eq.rhs, &known) > 0
        });

        if remaining.is_empty() {
            break;
        }

        // Step 1: find an equation with exactly one unknown
        let target_idx = remaining.iter().position(|eq| {
            let lhs_unknowns = count_unknowns(&eq.lhs, &known);
            let rhs_unknowns = count_unknowns(&eq.rhs, &known);
            lhs_unknowns + rhs_unknowns == 1
        });

        match target_idx {
            None => {
                // No progress possible — check if unknowns remain
                let has_unknowns = remaining.iter().any(|eq| {
                    count_unknowns(&eq.lhs, &known) + count_unknowns(&eq.rhs, &known) > 0
                });
                if has_unknowns {
                    return Err(SolverError::InsufficientInformation);
                }
                break;
            }
            Some(idx) => {
                let eq = remaining[idx].clone();

                // Step 2 & 3: find the unknown variable and isolate it
                let all_unknowns: Vec<String> = {
                    let mut v = collect_unknowns(&eq.lhs, &known);
                    v.extend(collect_unknowns(&eq.rhs, &known));
                    v.sort();
                    v.dedup();
                    v
                };

                let var = &all_unknowns[0];
                let value = isolate(&eq, var, &known)?;

                // Record step in human-readable form
                steps.push(format!("{} = {}", var, value));
                known.insert(var.clone(), value);

                // Step 4: substitute into all remaining equations
                substitute_all(&mut remaining, &known);

                // Remove the solved equation
                remaining.remove(idx);
            }
        }
    }

    Ok(Solution { values: known, steps })
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Equation, Expr, Operator};

    fn var(s: &str)  -> Expr { Expr::Variable(s.to_string()) }
    fn num(v: f64)   -> Expr { Expr::Number(v) }
    fn binop(op: Operator, l: Expr, r: Expr) -> Expr {
        Expr::BinaryOp { op, left: Box::new(l), right: Box::new(r) }
    }
    fn eq(lhs: Expr, rhs: Expr) -> Equation { Equation::new(lhs, rhs) }

    /// Two-variable system:
    ///   x = y + 3
    ///   y = 4
    /// Expected: y=4, x=7
    #[test]
    fn test_two_variable_system() {
        let equations = vec![
            eq(var("x"), binop(Operator::Add, var("y"), num(3.0))),
            eq(var("y"), num(4.0)),
        ];
        let sol = solve(equations).unwrap();
        assert_eq!(sol.values["y"], 4.0);
        assert_eq!(sol.values["x"], 7.0);
    }

    /// Three-variable chain:
    ///   a = 2
    ///   b = a * 3      → b = 6
    ///   c = b + a      → c = 8
    #[test]
    fn test_three_variable_chain() {
        let equations = vec![
            eq(var("a"), num(2.0)),
            eq(var("b"), binop(Operator::Mul, var("a"), num(3.0))),
            eq(var("c"), binop(Operator::Add, var("b"), var("a"))),
        ];
        let sol = solve(equations).unwrap();
        assert_eq!(sol.values["a"], 2.0);
        assert_eq!(sol.values["b"], 6.0);
        assert_eq!(sol.values["c"], 8.0);
    }

    /// Division by zero: x = 10 / 0  →  DivisionByZero
    #[test]
    fn test_division_by_zero() {
        let equations = vec![
            eq(var("x"), binop(Operator::Div, num(10.0), num(0.0))),
        ];
        let result = solve(equations);
        assert_eq!(result, Err(SolverError::DivisionByZero));
    }

    /// Insufficient information: x + y = 10 with no other constraint.
    #[test]
    fn test_insufficient_information() {
        let equations = vec![
            eq(binop(Operator::Add, var("x"), var("y")), num(10.0)),
        ];
        let result = solve(equations);
        assert_eq!(result, Err(SolverError::InsufficientInformation));
    }

    /// Quadratic rejection: x * x = 9  →  UnsupportedDegree(2)
    #[test]
    fn test_quadratic_rejected() {
        let equations = vec![
            eq(binop(Operator::Mul, var("x"), var("x")), num(9.0)),
        ];
        let result = solve(equations);
        assert_eq!(result, Err(SolverError::UnsupportedDegree(2)));
    }

    /// Unsupported domain: variable named "sin_theta" triggers domain check.
    #[test]
    fn test_unsupported_domain_rejected() {
        let equations = vec![
            eq(var("sin_theta"), num(1.0)),
        ];
        let result = solve(equations);
        assert_eq!(result, Err(SolverError::UnsupportedDomain("sin_theta".into())));
    }

    /// Solve steps are recorded in order.
    #[test]
    fn test_solution_steps_recorded() {
        let equations = vec![
            eq(var("x"), num(5.0)),
            eq(var("y"), binop(Operator::Add, var("x"), num(2.0))),
        ];
        let sol = solve(equations).unwrap();
        // Steps should mention x and y
        assert!(sol.steps.iter().any(|s| s.contains("x = 5")));
        assert!(sol.steps.iter().any(|s| s.contains("y = 7")));
    }

    /// Subtraction isolation: x - 3 = 7  →  x = 10
    #[test]
    fn test_subtraction_isolation() {
        let equations = vec![
            eq(binop(Operator::Sub, var("x"), num(3.0)), num(7.0)),
        ];
        let sol = solve(equations).unwrap();
        assert_eq!(sol.values["x"], 10.0);
    }

    /// Multiplication isolation: x * 4 = 20  →  x = 5
    #[test]
    fn test_multiplication_isolation() {
        let equations = vec![
            eq(binop(Operator::Mul, var("x"), num(4.0)), num(20.0)),
        ];
        let sol = solve(equations).unwrap();
        assert_eq!(sol.values["x"], 5.0);
    }
}
