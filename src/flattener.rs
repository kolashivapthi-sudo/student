use crate::types::{Equation, Expr};

// ---------------------------------------------------------------------------
// Flattener
// ---------------------------------------------------------------------------
// Goal: take a Vec<Equation> where the rhs may be a deeply nested BinaryOp
// tree, and produce a flat Vec<Equation> where every rhs is either:
//   - Expr::Number
//   - Expr::Variable
//   - A BinaryOp whose BOTH operands are leaves (Number or Variable)
//
// Nested sub-expressions are lifted into fresh intermediate variables
// named "_t0", "_t1", … and prepended as their own equations.
//
// Example input (one equation):
//   result = Add( Mul(x, 3), Add(y, 2) )
//
// Example flat output (three equations):
//   _t0 = x * 3
//   _t1 = y + 2
//   result = _t0 + _t1

// ---------------------------------------------------------------------------
// Counter for fresh intermediate variable names
// ---------------------------------------------------------------------------

struct TempCounter(usize);

impl TempCounter {
    fn new() -> Self { TempCounter(0) }

    fn next(&mut self) -> String {
        let name = format!("_t{}", self.0);
        self.0 += 1;
        name
    }
}

// ---------------------------------------------------------------------------
// Core recursive flatten
// ---------------------------------------------------------------------------

/// Flatten a single `Expr` into a sequence of flat equations.
///
/// If `expr` is already a leaf (Number / Variable) it is returned as-is
/// with no extra equations emitted.
///
/// If `expr` is a BinaryOp:
///   1. Recursively flatten left and right operands.
///   2. If an operand was itself a BinaryOp, it will have been replaced by
///      a fresh temp variable (the equation for it is in `out`).
///   3. Emit one equation: `_tN = flat_left OP flat_right`.
///   4. Return `Expr::Variable("_tN")` as the "handle" for the caller.
fn flatten_expr(expr: Expr, counter: &mut TempCounter, out: &mut Vec<Equation>) -> Expr {
    match expr {
        // Leaves are already flat — nothing to do.
        Expr::Number(_) | Expr::Variable(_) => expr,

        Expr::BinaryOp { op, left, right } => {
            // Recursively flatten each operand
            let flat_left  = flatten_expr(*left,  counter, out);
            let flat_right = flatten_expr(*right, counter, out);

            // Both operands are now leaves — build the flat binary node
            let flat_node = Expr::BinaryOp {
                op,
                left:  Box::new(flat_left),
                right: Box::new(flat_right),
            };

            // Introduce a temp variable to name this sub-result
            let temp = counter.next();
            out.push(Equation::new(
                Expr::Variable(temp.clone()),
                flat_node,
            ));

            // Return the temp variable as the handle for the parent
            Expr::Variable(temp)
        }
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Flatten a `Vec<Equation>` produced by `translator::translate()`.
///
/// Each equation is processed independently:
/// - If the rhs is already a flat leaf, the equation is kept as-is.
/// - If the rhs is a single-level BinaryOp with leaf operands, kept as-is
///   (already solvable in one step — no temp needed).
/// - If the rhs is a deeper BinaryOp, sub-expressions are lifted into
///   intermediate equations with temp variables `_t0`, `_t1`, …
///
/// The original variable assignment is always the LAST equation in the
/// output group for that input equation, so the solver sees the temp
/// definitions before the final assignment.
pub fn flatten(equations: Vec<Equation>) -> Vec<Equation> {
    let mut counter = TempCounter::new();
    let mut result:  Vec<Equation> = Vec::new();

    for eq in equations {
        // Check if rhs is already shallow enough (leaf or one-level BinaryOp)
        if is_flat(&eq.rhs) {
            result.push(eq);
            continue;
        }

        // Deeply nested — flatten rhs, collecting intermediate equations
        let mut intermediates: Vec<Equation> = Vec::new();
        let flat_rhs = flatten_expr(eq.rhs, &mut counter, &mut intermediates);

        // The last intermediate IS the rhs result — reassign to original lhs
        // instead of emitting a redundant temp-equals-temp equation.
        if let Some(last) = intermediates.last_mut() {
            last.lhs = eq.lhs;
        } else {
            // flatten_expr returned a leaf directly (shouldn't happen for
            // BinaryOp, but guard it)
            result.push(Equation::new(eq.lhs, flat_rhs));
            continue;
        }

        result.extend(intermediates);
    }

    result
}

/// Returns true if `expr` is a leaf or a one-level BinaryOp with leaf operands.
fn is_flat(expr: &Expr) -> bool {
    match expr {
        Expr::Number(_) | Expr::Variable(_) => true,
        Expr::BinaryOp { left, right, .. } => {
            is_leaf(left) && is_leaf(right)
        }
    }
}

/// Returns true if `expr` is a Number or Variable (a leaf node).
fn is_leaf(expr: &Expr) -> bool {
    matches!(expr, Expr::Number(_) | Expr::Variable(_))
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Equation, Expr, Operator};

    // Helpers
    fn var(s: &str)  -> Expr { Expr::Variable(s.to_string()) }
    fn num(v: f64)   -> Expr { Expr::Number(v) }
    fn binop(op: Operator, l: Expr, r: Expr) -> Expr {
        Expr::BinaryOp { op, left: Box::new(l), right: Box::new(r) }
    }
    fn eq(lhs: Expr, rhs: Expr) -> Equation { Equation::new(lhs, rhs) }

    /// A leaf rhs passes through unchanged.
    #[test]
    fn test_leaf_rhs_unchanged() {
        let input = vec![eq(var("x"), num(5.0))];
        let flat  = flatten(input);
        assert_eq!(flat, vec![eq(var("x"), num(5.0))]);
    }

    /// A one-level BinaryOp (both operands are leaves) passes through unchanged.
    #[test]
    fn test_single_level_binop_unchanged() {
        // x = y + 3  →  already flat
        let input = vec![eq(var("x"), binop(Operator::Add, var("y"), num(3.0)))];
        let flat  = flatten(input.clone());
        assert_eq!(flat, input);
    }

    /// Two-level nesting: rhs = (a + b) * c
    /// Expected flat output:
    ///   _t0 = a + b
    ///   result = _t0 * c
    #[test]
    fn test_two_level_nesting() {
        // result = (a + b) * c
        let nested = binop(
            Operator::Mul,
            binop(Operator::Add, var("a"), var("b")),   // nested left
            var("c"),
        );
        let input = vec![eq(var("result"), nested)];
        let flat  = flatten(input);

        assert_eq!(flat.len(), 2);
        // First equation: _t0 = a + b
        assert_eq!(flat[0].lhs, var("_t0"));
        assert_eq!(flat[0].rhs, binop(Operator::Add, var("a"), var("b")));
        // Second equation: result = _t0 * c
        assert_eq!(flat[1].lhs, var("result"));
        assert_eq!(flat[1].rhs, binop(Operator::Mul, var("_t0"), var("c")));
    }

    /// Three-level nesting: result = ((a + b) * c) - d
    /// Expected flat output:
    ///   _t0 = a + b
    ///   _t1 = _t0 * c
    ///   result = _t1 - d
    #[test]
    fn test_three_level_nesting() {
        let nested = binop(
            Operator::Sub,
            binop(
                Operator::Mul,
                binop(Operator::Add, var("a"), var("b")),  // level 3
                var("c"),
            ),
            var("d"),
        );
        let input = vec![eq(var("result"), nested)];
        let flat  = flatten(input);

        assert_eq!(flat.len(), 3);
        assert_eq!(flat[0].lhs, var("_t0"));
        assert_eq!(flat[0].rhs, binop(Operator::Add, var("a"), var("b")));

        assert_eq!(flat[1].lhs, var("_t1"));
        assert_eq!(flat[1].rhs, binop(Operator::Mul, var("_t0"), var("c")));

        assert_eq!(flat[2].lhs, var("result"));
        assert_eq!(flat[2].rhs, binop(Operator::Sub, var("_t1"), var("d")));
    }

    /// Multiple equations are each flattened independently, temp counter
    /// continues across equations so names never collide.
    #[test]
    fn test_multiple_equations_independent() {
        // eq1: x = (a + b) * 2   →  _t0 = a+b,  x = _t0 * 2   (uses _t0, _t1)
        // eq2: y = (c - d) + 1   →  _t2 = c-d,  y = _t2 + 1   (counter at 2)
        let eq1 = eq(
            var("x"),
            binop(Operator::Mul, binop(Operator::Add, var("a"), var("b")), num(2.0)),
        );
        let eq2 = eq(
            var("y"),
            binop(Operator::Add, binop(Operator::Sub, var("c"), var("d")), num(1.0)),
        );
        let flat = flatten(vec![eq1, eq2]);

        assert_eq!(flat.len(), 4);
        // eq1 intermediates
        assert_eq!(flat[0].lhs, var("_t0"));
        assert_eq!(flat[1].lhs, var("x"));
        // eq2 intermediates — counter continued, so _t2 (not _t1)
        assert_eq!(flat[2].lhs, var("_t2"));
        assert_eq!(flat[3].lhs, var("y"));
    }

    /// Mixed: first equation is already flat, second is nested.
    #[test]
    fn test_mixed_flat_and_nested() {
        let flat_eq  = eq(var("a"), num(10.0));
        let nested_eq = eq(
            var("b"),
            binop(Operator::Mul, binop(Operator::Add, var("a"), num(5.0)), num(2.0)),
        );
        let result = flatten(vec![flat_eq.clone(), nested_eq]);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], flat_eq);          // passed through unchanged
        assert_eq!(result[1].lhs, var("_t0"));   // intermediate
        assert_eq!(result[2].lhs, var("b"));     // final assignment
    }
}
