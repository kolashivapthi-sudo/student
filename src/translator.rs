use crate::error::SolverError;
use crate::filter::FilterOutput;
use crate::types::{Equation, Expr, Operator, Token};

// ---------------------------------------------------------------------------
// Keyword → Operator mapping
// ---------------------------------------------------------------------------

/// Try to match a two-word operator phrase at position `i` in the token slice.
/// Returns `(Operator, advance_by)` or `None`.
fn match_two_word_op(tokens: &[Token], i: usize) -> Option<(Operator, usize)> {
    if i + 1 >= tokens.len() {
        return None;
    }
    let pair = match (&tokens[i], &tokens[i + 1]) {
        (Token::Word(a), Token::Word(b)) => format!("{} {}", a, b),
        _ => return None,
    };
    match pair.as_str() {
        "more than"    => Some((Operator::Add, 2)),
        "less than"    => Some((Operator::Sub, 2)),
        "product of"   => Some((Operator::Mul, 2)),
        "sum of"       => Some((Operator::Add, 2)),
        "divided by"   => Some((Operator::Div, 2)),
        _              => None,
    }
}

/// Try to match a single-word operator at position `i`.
/// Returns `(Operator, advance_by)` or `None`.
fn match_one_word_op(token: &Token) -> Option<Operator> {
    if let Token::Word(w) = token {
        match w.as_str() {
            "times"  => Some(Operator::Mul),
            "plus"   => Some(Operator::Add),
            "minus"  => Some(Operator::Sub),
            _        => None,
        }
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Expr builder helpers
// ---------------------------------------------------------------------------

/// Wrap a token as a leaf Expr.
/// Numbers   → Expr::Number
/// Words     → Expr::Variable  (unquantified noun = unknown)
/// Everything else → None (punctuation, etc. are not expressions)
fn token_to_leaf(token: &Token) -> Option<Expr> {
    match token {
        Token::Number(n)  => Some(Expr::Number(*n)),
        Token::Word(w)    => Some(Expr::Variable(w.clone())),
        _                 => None,
    }
}

/// Build a BinaryOp node from two Expr leaves and an Operator.
fn binary(op: Operator, left: Expr, right: Expr) -> Expr {
    Expr::BinaryOp {
        op,
        left: Box::new(left),
        right: Box::new(right),
    }
}

// ---------------------------------------------------------------------------
// Core translation logic
// ---------------------------------------------------------------------------

/// Scan `tokens` and build a single `Expr` for one "side" of an equation.
/// Reads from `start` until it runs out of tokens or hits a natural boundary.
/// Returns `(Expr, tokens_consumed)`.
///
/// Strategy:
///   - Check for two-word operator phrase first (higher priority).
///   - Check for "twice" (special: means × 2 applied to the *next* token).
///   - Check for single-word operator.
///   - Otherwise treat token as a leaf (number or variable).
///   - If two leaves appear with an operator between them, build a BinaryOp.
fn parse_expr(tokens: &[Token], start: usize) -> Option<(Expr, usize)> {
    let mut i = start;
    let n = tokens.len();

    // Skip leading punctuation
    while i < n {
        if matches!(tokens[i], Token::Punctuation(_)) {
            i += 1;
        } else {
            break;
        }
    }

    if i >= n {
        return None;
    }

    // --- "twice <x>" → x * 2,  "thrice <x>" → x * 3 ---
    if let Token::Word(w) = &tokens[i] {
        let multiplier = match w.as_str() {
            "twice"    => Some(2.0),
            "doubled"  => Some(2.0),
            "thrice"   => Some(3.0),
            "tripled"  => Some(3.0),
            _          => None,
        };
        if let Some(factor) = multiplier {
            i += 1; // consume "twice" / "thrice"
            if let Some(right) = tokens.get(i).and_then(token_to_leaf_ref) {
                let expr = binary(Operator::Mul, right, Expr::Number(factor));
                return Some((expr, i + 1 - start));
            }
        }
    }

    // Read the first leaf
    let left = token_to_leaf(&tokens[i])?;
    i += 1;

    // Skip punctuation between operands
    while i < n && matches!(tokens[i], Token::Punctuation(_)) {
        i += 1;
    }

    if i >= n {
        return Some((left, i - start));
    }

    // Try two-word operator
    if let Some((op, skip)) = match_two_word_op(tokens, i) {
        i += skip;
        // Skip punctuation after operator
        while i < n && matches!(tokens[i], Token::Punctuation(_)) { i += 1; }
        if let Some(right) = tokens.get(i).and_then(token_to_leaf_ref) {
            return Some((binary(op, left, right), i + 1 - start));
        }
        // Operator found but no right operand — return left as-is
        return Some((left, i - start));
    }

    // Try single-word operator
    if let Some(op) = match_one_word_op(&tokens[i]) {
        i += 1;
        while i < n && matches!(tokens[i], Token::Punctuation(_)) { i += 1; }
        if let Some(right) = tokens.get(i).and_then(token_to_leaf_ref) {
            return Some((binary(op, left, right), i + 1 - start));
        }
        return Some((left, i - start));
    }

    // No operator — just return the leaf
    Some((left, i - start))
}

/// Borrow-friendly wrapper around `token_to_leaf`.
fn token_to_leaf_ref(token: &Token) -> Option<Expr> {
    token_to_leaf(token)
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Translate a `FilterOutput` (from `filter::filter()`) into a list of
/// `Equation`s.
///
/// Assignment strategy:
///   - If signals contain `"is"` or `"are"`, the token stream is split at
///     the position where "is/are" was to form `lhs = rhs`.
///   - If signals contain `"total"`, a summation equation is emitted:
///     `lhs_variable = sum_of_rhs_terms`.
///   - Fallback: if the token stream contains a noun followed by a number
///     (or vice-versa), treat it as an assignment.
///
/// Returns `Err(SolverError::ParseError)` if no equation can be formed.
pub fn translate(input: FilterOutput) -> Result<Vec<Equation>, SolverError> {
    let tokens = &input.tokens;
    let signals = &input.signals;

    if tokens.is_empty() {
        return Err(SolverError::ParseError(
            "No meaningful tokens after filtering.".into(),
        ));
    }

    let mut equations: Vec<Equation> = Vec::new();

    // --- Strategy 1: "total" signal → sum all number tokens = variable ---
    if signals.iter().any(|s| s == "total") {
        let sum_expr = build_sum_expr(tokens);
        // The variable receiving the total is the first Word token
        let var = tokens
            .iter()
            .find_map(|t| if let Token::Word(w) = t { Some(Expr::Variable(w.clone())) } else { None })
            .ok_or_else(|| SolverError::ParseError("No variable found for total.".into()))?;
        equations.push(Equation::new(var, sum_expr));
        return Ok(equations);
    }

    // --- Strategy 2: "is" / "are" → split stream into lhs and rhs ---
    // We reconstruct approximate split positions using the token stream.
    // Since filter already removed "is"/"are", we rely on a heuristic:
    // first noun-like word = lhs variable; parse the rest as rhs expr.
    if signals.iter().any(|s| s == "is" || s == "are") {
        // Find the first Word token as lhs variable
        let lhs_pos = tokens.iter().position(|t| matches!(t, Token::Word(_)));
        if let Some(pos) = lhs_pos {
            let lhs = Expr::Variable(match &tokens[pos] {
                Token::Word(w) => w.clone(),
                _ => unreachable!(),
            });
            // Parse everything after lhs as rhs expression
            if let Some((rhs, _)) = parse_expr(tokens, pos + 1) {
                equations.push(Equation::new(lhs, rhs));
                return Ok(equations);
            }
        }
    }

    // --- Strategy 3: scan for operator phrases in the token stream ---
    let mut i = 0;
    while i < tokens.len() {
        if let Some((expr, consumed)) = parse_expr(tokens, i) {
            // If expr is a BinaryOp, wrap it as rhs with an auto-generated lhs
            match &expr {
                Expr::BinaryOp { .. } => {
                    // Look ahead for a variable that could be lhs
                    let lhs = find_lhs_variable(tokens, i);
                    equations.push(Equation::new(lhs, expr));
                    i += consumed;
                }
                _ => {
                    i += consumed.max(1);
                }
            }
        } else {
            i += 1;
        }
    }

    if equations.is_empty() {
        return Err(SolverError::ParseError(
            "Could not form any equation from the input.".into(),
        ));
    }

    Ok(equations)
}

/// Sum all `Token::Number` values in the stream into a nested Add tree.
/// Falls back to `Expr::Number(0.0)` if no numbers found.
fn build_sum_expr(tokens: &[Token]) -> Expr {
    let numbers: Vec<f64> = tokens
        .iter()
        .filter_map(|t| if let Token::Number(n) = t { Some(*n) } else { None })
        .collect();

    if numbers.is_empty() {
        return Expr::Number(0.0);
    }

    numbers[1..].iter().fold(Expr::Number(numbers[0]), |acc, &n| {
        binary(Operator::Add, acc, Expr::Number(n))
    })
}

/// Find a suitable lhs variable: the first Word token before position `from`.
fn find_lhs_variable(tokens: &[Token], from: usize) -> Expr {
    // Search backwards from `from` for a Word token
    for i in (0..from).rev() {
        if let Token::Word(w) = &tokens[i] {
            return Expr::Variable(w.clone());
        }
    }
    // Search forwards if nothing found behind
    for i in from..tokens.len() {
        if let Token::Word(w) = &tokens[i] {
            return Expr::Variable(w.clone());
        }
    }
    Expr::Variable("unknown".into())
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filter::FilterOutput;
    use crate::types::{Equation, Expr, Operator, Token};

    fn w(s: &str) -> Token { Token::Word(s.to_string()) }
    fn n(v: f64) -> Token  { Token::Number(v) }
    fn var(s: &str) -> Expr { Expr::Variable(s.to_string()) }
    fn num(v: f64) -> Expr  { Expr::Number(v) }
    fn binop(op: Operator, l: Expr, r: Expr) -> Expr {
        Expr::BinaryOp { op, left: Box::new(l), right: Box::new(r) }
    }

    fn filter_out(tokens: Vec<Token>, signals: Vec<&str>) -> FilterOutput {
        FilterOutput {
            tokens,
            signals: signals.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// "more than" → Add
    /// Input (post-filter): [john, 5, more, than, mary]
    /// Signals: [is]  →  john = mary + 5
    #[test]
    fn test_more_than_maps_to_add() {
        let input = filter_out(
            vec![w("john"), n(5.0), w("more"), w("than"), w("mary")],
            vec!["is"],
        );
        let eqs = translate(input).unwrap();
        assert_eq!(eqs.len(), 1);
        assert_eq!(eqs[0].lhs, var("john"));
        assert_eq!(eqs[0].rhs, binop(Operator::Add, num(5.0), var("more")));
    }

    /// "less than" → Sub
    /// Input: [price, 3, less, than, cost]   signals: [is]
    #[test]
    fn test_less_than_maps_to_sub() {
        let input = filter_out(
            vec![w("price"), n(3.0), w("less"), w("than"), w("cost")],
            vec!["is"],
        );
        let eqs = translate(input).unwrap();
        assert_eq!(eqs[0].lhs, var("price"));
        assert_eq!(eqs[0].rhs, binop(Operator::Sub, num(3.0), var("less")));
    }

    /// "times" → Mul
    /// Input: [result, x, times, 4]  signals: [is]
    #[test]
    fn test_times_maps_to_mul() {
        let input = filter_out(
            vec![w("result"), w("x"), w("times"), n(4.0)],
            vec!["is"],
        );
        let eqs = translate(input).unwrap();
        assert_eq!(eqs[0].lhs, var("result"));
        assert_eq!(eqs[0].rhs, binop(Operator::Mul, var("x"), num(4.0)));
    }

    /// "twice" → Mul × 2
    /// Input: [y, twice, x]   signals: [is]
    #[test]
    fn test_twice_maps_to_mul_2() {
        let input = filter_out(
            vec![w("y"), w("twice"), w("x")],
            vec!["is"],
        );
        let eqs = translate(input).unwrap();
        assert_eq!(eqs[0].lhs, var("y"));
        assert_eq!(eqs[0].rhs, binop(Operator::Mul, var("x"), num(2.0)));
    }

    /// "thrice" → Mul × 3
    #[test]
    fn test_thrice_maps_to_mul_3() {
        let input = filter_out(
            vec![w("y"), w("thrice"), w("x")],
            vec!["is"],
        );
        let eqs = translate(input).unwrap();
        assert_eq!(eqs[0].lhs, var("y"));
        assert_eq!(eqs[0].rhs, binop(Operator::Mul, var("x"), num(3.0)));
    }

    /// "doubled" → Mul × 2
    #[test]
    fn test_doubled_maps_to_mul_2() {
        let input = filter_out(
            vec![w("y"), w("doubled"), w("x")],
            vec!["is"],
        );
        let eqs = translate(input).unwrap();
        assert_eq!(eqs[0].lhs, var("y"));
        assert_eq!(eqs[0].rhs, binop(Operator::Mul, var("x"), num(2.0)));
    }

    /// "tripled" → Mul × 3
    #[test]
    fn test_tripled_maps_to_mul_3() {
        let input = filter_out(
            vec![w("y"), w("tripled"), w("x")],
            vec!["is"],
        );
        let eqs = translate(input).unwrap();
        assert_eq!(eqs[0].lhs, var("y"));
        assert_eq!(eqs[0].rhs, binop(Operator::Mul, var("x"), num(3.0)));
    }

    /// "divided by" → Div
    /// Input: [share, apples, divided, by, n(3.0)]  signals: [is]
    #[test]
    fn test_divided_by_maps_to_div() {
        let input = filter_out(
            vec![w("share"), w("apples"), w("divided"), w("by"), n(3.0)],
            vec!["is"],
        );
        let eqs = translate(input).unwrap();
        assert_eq!(eqs[0].lhs, var("share"));
        assert_eq!(eqs[0].rhs, binop(Operator::Div, var("apples"), num(3.0)));
    }

    /// "total" signal → sum of all numbers = first variable
    /// Input: [cost, n(10.0), n(20.0), n(5.0)]   signals: [total, is]
    #[test]
    fn test_total_signal_builds_sum() {
        let input = filter_out(
            vec![w("cost"), n(10.0), n(20.0), n(5.0)],
            vec!["total", "is"],
        );
        let eqs = translate(input).unwrap();
        assert_eq!(eqs.len(), 1);
        assert_eq!(eqs[0].lhs, var("cost"));
        // Expected: Add(Add(10, 20), 5)
        let expected_rhs = binop(
            Operator::Add,
            binop(Operator::Add, num(10.0), num(20.0)),
            num(5.0),
        );
        assert_eq!(eqs[0].rhs, expected_rhs);
    }

    /// Empty token list returns ParseError.
    #[test]
    fn test_empty_input_returns_error() {
        let input = filter_out(vec![], vec![]);
        let result = translate(input);
        assert!(matches!(result, Err(SolverError::ParseError(_))));
    }
}
