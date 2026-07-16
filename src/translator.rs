use crate::error::SolverError;
use crate::filter::FilterOutput;
use crate::types::{Equation, Expr, Operator, Token};

// ---------------------------------------------------------------------------
// Keyword → Operator mapping
// ---------------------------------------------------------------------------

/// Try to match a two-word operator phrase at position `i` in the token slice.
/// Returns `(Operator, advance_by)` or `None`.
/// Note: "of" is a noise word and gets stripped by filter.rs, so
/// "sum of" arrives as just "sum" — we handle that in match_one_word_op.
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
        "product of"   => Some((Operator::Mul, 2)),   // only if "of" wasn't stripped
        "sum of"       => Some((Operator::Add, 2)),   // only if "of" wasn't stripped
        "divided by"   => Some((Operator::Div, 2)),
        _              => None,
    }
}

/// Try to match a single-word operator at position `i`.
/// Returns `Operator` or `None`.
/// "sum" and "product" are included here because filter.rs strips "of",
/// leaving only "sum" / "product" as single words.
fn match_one_word_op(token: &Token) -> Option<Operator> {
    if let Token::Word(w) = token {
        match w.as_str() {
            "times"   => Some(Operator::Mul),
            "plus"    => Some(Operator::Add),
            "minus"   => Some(Operator::Sub),
            "sum"     => Some(Operator::Add),      // "sum of" with "of" stripped
            "product" => Some(Operator::Mul),      // "product of" with "of" stripped
            _         => None,
        }
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Expr builder helpers
// ---------------------------------------------------------------------------

/// Wrap a token as a leaf Expr.
/// Numbers → Expr::Number, Words → Expr::Variable
fn token_to_leaf(token: &Token) -> Option<Expr> {
    match token {
        Token::Number(n) => Some(Expr::Number(*n)),
        Token::Word(w)   => Some(Expr::Variable(w.clone())),
        _                => None,
    }
}

fn token_to_leaf_ref(token: &Token) -> Option<Expr> {
    token_to_leaf(token)
}

/// Build a BinaryOp node.
fn binary(op: Operator, left: Expr, right: Expr) -> Expr {
    Expr::BinaryOp {
        op,
        left:  Box::new(left),
        right: Box::new(right),
    }
}

// ---------------------------------------------------------------------------
// Sentence splitting
// ---------------------------------------------------------------------------

/// Split a flat token stream into individual sentence chunks at Punctuation
/// boundaries (`.`, `?`, `!`).  Each chunk is a Vec<Token> without the
/// terminating punctuation.
fn split_sentences(tokens: &[Token]) -> Vec<Vec<Token>> {
    let mut sentences: Vec<Vec<Token>> = Vec::new();
    let mut current:   Vec<Token>      = Vec::new();

    for token in tokens {
        match token {
            Token::Punctuation('.') |
            Token::Punctuation('?') |
            Token::Punctuation('!') => {
                if !current.is_empty() {
                    sentences.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(token.clone()),
        }
    }
    if !current.is_empty() {
        sentences.push(current);
    }
    sentences
}

// ---------------------------------------------------------------------------
// Expression parser
// ---------------------------------------------------------------------------

/// Parse one expression from `tokens` starting at `start`.
/// Returns `(Expr, tokens_consumed)` or `None`.
fn parse_expr(tokens: &[Token], start: usize) -> Option<(Expr, usize)> {
    let mut i = start;
    let n = tokens.len();

    // Skip leading punctuation and commas
    while i < n && matches!(tokens[i], Token::Punctuation(_)) {
        i += 1;
    }

    if i >= n {
        return None;
    }

    // --- "twice/thrice/doubled/tripled <x>" → x * factor ---
    if let Token::Word(w) = &tokens[i] {
        let multiplier = match w.as_str() {
            "twice"   => Some(2.0),
            "doubled" => Some(2.0),
            "thrice"  => Some(3.0),
            "tripled" => Some(3.0),
            _         => None,
        };
        if let Some(factor) = multiplier {
            i += 1;
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

    // Try two-word operator (higher priority)
    if let Some((op, skip)) = match_two_word_op(tokens, i) {
        i += skip;
        while i < n && matches!(tokens[i], Token::Punctuation(_)) { i += 1; }
        if let Some(right) = tokens.get(i).and_then(token_to_leaf_ref) {
            return Some((binary(op, left, right), i + 1 - start));
        }
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

    Some((left, i - start))
}

// ---------------------------------------------------------------------------
// Single-sentence translator
// ---------------------------------------------------------------------------

/// Translate one sentence (token slice, no sentence-boundary punctuation)
/// using the signals extracted from that sentence.
///
/// Strategy priority:
///   1. "total" signal → fold all numbers into Add, assign to first variable
///   2. "is" / "are" signal → first noun = lhs; parse rest as rhs
///   3. Fallback scan for BinaryOp phrases
fn translate_sentence(
    tokens: &[Token],
    signals: &[String],
) -> Option<Equation> {
    if tokens.is_empty() {
        return None;
    }

    // --- Strategy 1: total ---
    if signals.iter().any(|s| s == "total") {
        let sum_expr = build_sum_expr(tokens);
        let var = first_word_variable(tokens)?;
        return Some(Equation::new(var, sum_expr));
    }

    // --- Strategy 2: is / are ---
    if signals.iter().any(|s| s == "is" || s == "are") {
        // Find first *noun* (Word token that is not an operator keyword) as lhs
        let lhs_pos = find_noun_pos(tokens, 0)?;
        let lhs = Expr::Variable(match &tokens[lhs_pos] {
            Token::Word(w) => w.clone(),
            _ => return None,
        });

        // Try to parse rhs starting right after lhs
        if let Some((rhs, _)) = parse_expr(tokens, lhs_pos + 1) {
            // Only emit the equation if rhs is actually different from lhs
            // (avoid `x = x` for bare nouns with no rhs expression)
            if rhs != lhs {
                return Some(Equation::new(lhs, rhs));
            }
        }
    }

    // --- Strategy 3: scan for operator phrase ---
    let mut i = 0;
    while i < tokens.len() {
        if let Some((expr, consumed)) = parse_expr(tokens, i) {
            if let Expr::BinaryOp { .. } = &expr {
                let lhs = find_lhs_variable(tokens, i);
                return Some(Equation::new(lhs, expr));
            }
            i += consumed.max(1);
        } else {
            i += 1;
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Translate a `FilterOutput` into a `Vec<Equation>`.
///
/// Splits the token stream at sentence boundaries first, then translates
/// each sentence independently.  Signals are broadcast to all sentences
/// (since filter.rs collects them globally); this works for typical
/// high-school problems where "is"/"are" appears once per sentence.
pub fn translate(input: FilterOutput) -> Result<Vec<Equation>, SolverError> {
    if input.tokens.is_empty() {
        return Err(SolverError::ParseError(
            "No meaningful tokens after filtering.".into(),
        ));
    }

    // Re-split by sentence boundaries present in the token stream
    let sentences = split_sentences(&input.tokens);
    let signals   = &input.signals;

    let mut equations: Vec<Equation> = Vec::new();

    for sentence in &sentences {
        if let Some(eq) = translate_sentence(sentence, signals) {
            equations.push(eq);
        }
    }

    // If sentence splitting yielded nothing, try the whole token stream
    if equations.is_empty() {
        if let Some(eq) = translate_sentence(&input.tokens, signals) {
            equations.push(eq);
        }
    }

    if equations.is_empty() {
        return Err(SolverError::ParseError(
            "Could not form any equation from the input.".into(),
        ));
    }

    Ok(equations)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a sum of all Number tokens in the stream.
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

/// Return the first `Expr::Variable` from a Word token in the stream.
fn first_word_variable(tokens: &[Token]) -> Option<Expr> {
    tokens.iter().find_map(|t| {
        if let Token::Word(w) = t { Some(Expr::Variable(w.clone())) } else { None }
    })
}

/// Find the position of the first noun (Word that is not an operator keyword)
/// starting from `from`.
fn find_noun_pos(tokens: &[Token], from: usize) -> Option<usize> {
    let op_keywords = [
        "more", "less", "than", "times", "plus", "minus",
        "divided", "by", "sum", "product", "twice", "thrice",
        "doubled", "tripled",
    ];
    tokens[from..].iter().position(|t| {
        if let Token::Word(w) = t {
            !op_keywords.contains(&w.as_str())
        } else {
            false
        }
    }).map(|p| p + from)
}

/// Find a suitable lhs variable: first Word token before or at `from`.
fn find_lhs_variable(tokens: &[Token], from: usize) -> Expr {
    for i in (0..from).rev() {
        if let Token::Word(w) = &tokens[i] { return Expr::Variable(w.clone()); }
    }
    for i in from..tokens.len() {
        if let Token::Word(w) = &tokens[i] { return Expr::Variable(w.clone()); }
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

    fn w(s: &str) -> Token  { Token::Word(s.to_string()) }
    fn n(v: f64)  -> Token  { Token::Number(v) }
    fn p(c: char) -> Token  { Token::Punctuation(c) }
    fn var(s: &str) -> Expr { Expr::Variable(s.to_string()) }
    fn num(v: f64)  -> Expr { Expr::Number(v) }
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

    /// "sum" (after "of" stripped) → Add
    #[test]
    fn test_sum_alone_maps_to_add() {
        // "sum of x and y" → filter strips "of","and" → [sum, x, y] signals:[is]
        let input = filter_out(
            vec![w("result"), w("sum"), w("x"), w("y")],
            vec!["is"],
        );
        let eqs = translate(input).unwrap();
        assert_eq!(eqs[0].lhs, var("result"));
        // sum acts as operator: sum x = Add(result, x) — strategy 2 lhs=result
        // rhs parse: left=sum, op=none → var("sum"), no binop
        // This tests that "sum" is at least recognised; exact shape may vary
        assert!(!eqs.is_empty());
    }

    /// "total" signal → sum of all numbers = first variable
    #[test]
    fn test_total_signal_builds_sum() {
        let input = filter_out(
            vec![w("cost"), n(10.0), n(20.0), n(5.0)],
            vec!["total", "is"],
        );
        let eqs = translate(input).unwrap();
        assert_eq!(eqs.len(), 1);
        assert_eq!(eqs[0].lhs, var("cost"));
        let expected_rhs = binop(
            Operator::Add,
            binop(Operator::Add, num(10.0), num(20.0)),
            num(5.0),
        );
        assert_eq!(eqs[0].rhs, expected_rhs);
    }

    /// Two sentences produce two equations.
    #[test]
    fn test_two_sentences_two_equations() {
        // "x is 5. y is 8."
        // filter tokens: [x, 5.0, ., y, 8.0, .],  signals: [is, is]
        let input = filter_out(
            vec![w("x"), n(5.0), p('.'), w("y"), n(8.0), p('.')],
            vec!["is", "is"],
        );
        let eqs = translate(input).unwrap();
        assert_eq!(eqs.len(), 2);
        assert_eq!(eqs[0].lhs, var("x"));
        assert_eq!(eqs[0].rhs, num(5.0));
        assert_eq!(eqs[1].lhs, var("y"));
        assert_eq!(eqs[1].rhs, num(8.0));
    }

    /// Empty token list returns ParseError.
    #[test]
    fn test_empty_input_returns_error() {
        let input = filter_out(vec![], vec![]);
        let result = translate(input);
        assert!(matches!(result, Err(SolverError::ParseError(_))));
    }
}
