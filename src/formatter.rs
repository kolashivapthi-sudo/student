use std::collections::HashMap;
use crate::types::{Equation, Expr, Operator};

// ---------------------------------------------------------------------------
// Expr → readable string
// ---------------------------------------------------------------------------

/// Render an `Expr` as a human-readable algebraic string.
///
/// Examples:
///   Expr::Number(5.0)              →  "5"
///   Expr::Variable("john")         →  "john"
///   BinaryOp(Add, x, 3)            →  "x + 3"
///   BinaryOp(Mul, BinaryOp(..), y) →  "(x + 3) * y"   ← parens for nested
pub fn expr_to_string(expr: &Expr) -> String {
    match expr {
        Expr::Number(n) => {
            // Print as integer if the value is whole, else as decimal
            if n.fract() == 0.0 {
                format!("{}", *n as i64)
            } else {
                format!("{}", n)
            }
        }
        Expr::Variable(name) => name.clone(),

        Expr::BinaryOp { op, left, right } => {
            let op_str = op_symbol(op);
            let left_str  = expr_to_string_maybe_parens(left,  op, true);
            let right_str = expr_to_string_maybe_parens(right, op, false);
            format!("{} {} {}", left_str, op_str, right_str)
        }
    }
}

/// Render the operator as an ASCII symbol.
fn op_symbol(op: &Operator) -> &'static str {
    match op {
        Operator::Add => "+",
        Operator::Sub => "-",
        Operator::Mul => "*",
        Operator::Div => "/",
    }
}

/// Render a sub-expression, wrapping in parentheses if it is itself a
/// BinaryOp (to make precedence explicit and unambiguous).
fn expr_to_string_maybe_parens(expr: &Expr, _parent_op: &Operator, _is_left: bool) -> String {
    match expr {
        Expr::BinaryOp { .. } => format!("({})", expr_to_string(expr)),
        _ => expr_to_string(expr),
    }
}

/// Render a single `Equation` as "lhs = rhs".
pub fn equation_to_string(eq: &Equation) -> String {
    format!("{} = {}", expr_to_string(&eq.lhs), expr_to_string(&eq.rhs))
}

// ---------------------------------------------------------------------------
// Format the full output
// ---------------------------------------------------------------------------

/// The formatted output returned by `format_output`.
/// Keeps display string and raw parts separate so tests can inspect them.
#[derive(Debug, Clone)]
pub struct FormattedOutput {
    /// The complete printable string (what gets shown to the user).
    pub display: String,
    /// The equation lines shown in the "Expressions:" section.
    pub equation_lines: Vec<String>,
    /// The answer lines shown in the "Answer:" section.
    pub answer_lines: Vec<String>,
}

/// Build the formatted output from:
///   - `equations`  — the flat equation list (from flattener, shown as work)
///   - `steps`      — the solve trace from solver (e.g. ["x = 7", "y = 4"])
///   - `values`     — the final solved variable map
///   - `question_vars` — the variable names the user actually asked about
///                       (used to pick which answer line(s) to highlight)
///
/// Output format (matching the project spec):
///
///   Expressions:
///     x + 3 = 10
///     x = 7
///
///   Answer: x = 7
pub fn format_output(
    equations: &[Equation],
    steps: &[String],
    values: &HashMap<String, f64>,
    question_vars: &[String],
) -> FormattedOutput {
    let mut lines: Vec<String> = Vec::new();

    // --- Section 1: Expressions (the equations written for solving) ---
    lines.push("Expressions:".to_string());

    // Show the flat equations as written algebraic form
    let eq_lines: Vec<String> = equations
        .iter()
        // Filter out temp variables (_t0, _t1, …) from display — they are
        // internal scaffolding, not meaningful to the student.
        // Hide if: lhs is a temp var, OR rhs is solely a temp var reference.
        .filter(|eq| !is_temp_var(&eq.lhs) && !is_temp_var(&eq.rhs))
        .map(equation_to_string)
        .collect();

    for line in &eq_lines {
        lines.push(format!("  {}", line));
    }

    // If solver produced extra step lines not already in eq display, show them
    for step in steps {
        let already_shown = eq_lines.iter().any(|l| l == step);
        if !already_shown {
            lines.push(format!("  {}", step));
        }
    }

    lines.push(String::new()); // blank line

    // --- Section 2: Answer ---
    let answer_lines: Vec<String> = if question_vars.is_empty() {
        // No specific question vars — show all solved variables
        let mut sorted: Vec<(&String, &f64)> = values.iter().collect();
        sorted.sort_by_key(|(k, _)| k.as_str());
        sorted
            .iter()
            .filter(|(k, _)| !k.starts_with('_'))   // hide temps
            .map(|(k, v)| format!("{} = {}", k, fmt_number(**v)))
            .collect()
    } else {
        question_vars
            .iter()
            .filter_map(|var| {
                values.get(var).map(|v| format!("{} = {}", var, fmt_number(*v)))
            })
            .collect()
    };

    if answer_lines.len() == 1 {
        lines.push(format!("Answer: {}", answer_lines[0]));
    } else {
        lines.push("Answer:".to_string());
        for al in &answer_lines {
            lines.push(format!("  {}", al));
        }
    }

    let display = lines.join("\n");
    FormattedOutput { display, equation_lines: eq_lines, answer_lines }
}

/// Print the formatted output to stdout.
pub fn print_output(output: &FormattedOutput) {
    println!("{}", output.display);
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Format a number: drop the decimal point if the value is whole.
fn fmt_number(n: f64) -> String {
    if n.fract() == 0.0 {
        format!("{}", n as i64)
    } else {
        format!("{}", n)
    }
}

/// Returns true if the expression is a temp variable (_t0, _t1, …).
fn is_temp_var(expr: &Expr) -> bool {
    matches!(expr, Expr::Variable(name) if name.starts_with('_'))
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Equation, Expr, Operator};
    use std::collections::HashMap;

    fn var(s: &str)  -> Expr { Expr::Variable(s.to_string()) }
    fn num(v: f64)   -> Expr { Expr::Number(v) }
    fn binop(op: Operator, l: Expr, r: Expr) -> Expr {
        Expr::BinaryOp { op, left: Box::new(l), right: Box::new(r) }
    }
    fn eq(lhs: Expr, rhs: Expr) -> Equation { Equation::new(lhs, rhs) }

    // --- expr_to_string tests ---

    #[test]
    fn test_number_whole_prints_without_decimal() {
        assert_eq!(expr_to_string(&num(7.0)), "7");
    }

    #[test]
    fn test_number_decimal_prints_with_decimal() {
        assert_eq!(expr_to_string(&num(3.5)), "3.5");
    }

    #[test]
    fn test_variable_prints_name() {
        assert_eq!(expr_to_string(&var("john")), "john");
    }

    #[test]
    fn test_simple_binop_add() {
        let expr = binop(Operator::Add, var("x"), num(3.0));
        assert_eq!(expr_to_string(&expr), "x + 3");
    }

    #[test]
    fn test_nested_binop_adds_parens() {
        // (x + 3) * y
        let inner = binop(Operator::Add, var("x"), num(3.0));
        let outer = binop(Operator::Mul, inner, var("y"));
        assert_eq!(expr_to_string(&outer), "(x + 3) * y");
    }

    #[test]
    fn test_equation_to_string() {
        let equation = eq(var("x"), binop(Operator::Add, var("y"), num(3.0)));
        assert_eq!(equation_to_string(&equation), "x = y + 3");
    }

    // --- format_output tests ---

    /// Single variable solved: checks "Expressions:" section and "Answer:" line.
    #[test]
    fn test_format_single_variable() {
        let equations = vec![
            eq(var("x"), binop(Operator::Add, var("y"), num(3.0))),
            eq(var("y"), num(4.0)),
        ];
        let steps = vec!["y = 4".to_string(), "x = 7".to_string()];
        let mut values = HashMap::new();
        values.insert("x".to_string(), 7.0);
        values.insert("y".to_string(), 4.0);
        let question_vars = vec!["x".to_string()];

        let out = format_output(&equations, &steps, &values, &question_vars);

        assert!(out.display.contains("Expressions:"));
        assert!(out.display.contains("x = y + 3"));
        assert!(out.display.contains("Answer: x = 7"));
    }

    /// Multiple answer variables use the "Answer:" + indented format.
    #[test]
    fn test_format_multiple_answer_vars() {
        let equations = vec![
            eq(var("x"), num(5.0)),
            eq(var("y"), num(8.0)),
        ];
        let steps = vec!["x = 5".to_string(), "y = 8".to_string()];
        let mut values = HashMap::new();
        values.insert("x".to_string(), 5.0);
        values.insert("y".to_string(), 8.0);
        let question_vars = vec!["x".to_string(), "y".to_string()];

        let out = format_output(&equations, &steps, &values, &question_vars);

        assert!(out.display.contains("Answer:"));
        assert!(out.display.contains("  x = 5"));
        assert!(out.display.contains("  y = 8"));
    }

    /// Temp variables (_t0, _t1) are hidden from display.
    #[test]
    fn test_temp_variables_hidden() {
        let equations = vec![
            eq(var("_t0"), binop(Operator::Add, var("a"), var("b"))),
            eq(var("result"), var("_t0")),
        ];
        let steps = vec!["_t0 = 9".to_string(), "result = 9".to_string()];
        let mut values = HashMap::new();
        values.insert("_t0".to_string(), 9.0);
        values.insert("result".to_string(), 9.0);
        let question_vars = vec!["result".to_string()];

        let out = format_output(&equations, &steps, &values, &question_vars);

        // _t0 equation should not appear in expression lines
        assert!(!out.equation_lines.iter().any(|l| l.contains("_t0")));
        assert!(out.display.contains("Answer: result = 9"));
    }

    /// Whole number answers drop the decimal point.
    #[test]
    fn test_answer_whole_number_no_decimal() {
        let equations = vec![eq(var("x"), num(10.0))];
        let steps = vec!["x = 10".to_string()];
        let mut values = HashMap::new();
        values.insert("x".to_string(), 10.0);

        let out = format_output(&equations, &steps, &values, &["x".to_string()]);
        assert!(out.display.contains("Answer: x = 10"));
        assert!(!out.display.contains("10.0"));
    }
}
