use crate::types::Token;

// ---------------------------------------------------------------------------
// Word lists
// ---------------------------------------------------------------------------

/// Pure noise — carry no algebraic meaning, discard completely.
const NOISE_WORDS: &[&str] = &[
    "a", "an", "the", "was", "were", "if", "of", "and",
    "then", "each", "some", "there", "find", "how", "many", "what",
];

/// Signal words — grammatical noise BUT carry semantic meaning for
/// translator.rs (assignment "=" or summation cue).
/// These are removed from the token stream but returned separately
/// so the translator can use them.
const SIGNAL_WORDS: &[&str] = &["is", "are", "total"];

// ---------------------------------------------------------------------------
// Output type
// ---------------------------------------------------------------------------

/// Result of the filter stage.
///
/// - `tokens`  — the cleaned token stream with all noise/signal words removed.
/// - `signals` — the signal words that were stripped (preserves order and
///               duplicates so the translator knows how many assignment
///               cues appeared and where).
#[derive(Debug, Clone, PartialEq)]
pub struct FilterOutput {
    pub tokens: Vec<Token>,
    pub signals: Vec<String>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Filters a `Vec<Token>` produced by the lexer.
///
/// - Pure noise words are silently dropped.
/// - Signal words (`is`, `are`, `total`) are removed from the token stream
///   but collected into `FilterOutput::signals`.
/// - Numbers, Punctuation, and Operator tokens pass through unchanged.
/// - Non-noise/non-signal `Word` tokens pass through unchanged.
pub fn filter(tokens: Vec<Token>) -> FilterOutput {
    let mut clean: Vec<Token> = Vec::new();
    let mut signals: Vec<String> = Vec::new();

    for token in tokens {
        match &token {
            Token::Word(w) => {
                let word = w.as_str();
                if SIGNAL_WORDS.contains(&word) {
                    // Preserve as a signal, drop from main stream
                    signals.push(w.clone());
                } else if NOISE_WORDS.contains(&word) {
                    // Pure noise — discard
                } else {
                    clean.push(token);
                }
            }
            // Numbers, Operators, Punctuation always pass through
            _ => clean.push(token),
        }
    }

    FilterOutput { tokens: clean, signals }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Token;

    // Helper: build a Word token
    fn w(s: &str) -> Token { Token::Word(s.to_string()) }
    fn n(v: f64) -> Token  { Token::Number(v) }
    fn p(c: char) -> Token { Token::Punctuation(c) }

    /// Pure noise words are removed, meaningful words survive.
    #[test]
    fn test_noise_words_removed() {
        let input = vec![
            w("john"), w("has"), w("a"), w("bag"), w("of"), w("apples"),
        ];
        let out = filter(input);
        assert_eq!(
            out.tokens,
            vec![w("john"), w("has"), w("bag"), w("apples")]
        );
        assert!(out.signals.is_empty());
    }

    /// Signal words are removed from token stream but appear in signals list.
    #[test]
    fn test_signal_words_preserved_separately() {
        let input = vec![
            w("john"), w("is"), w("older"), w("than"), w("mary"),
        ];
        let out = filter(input);
        assert_eq!(
            out.tokens,
            vec![w("john"), w("older"), w("than"), w("mary")]
        );
        assert_eq!(out.signals, vec!["is".to_string()]);
    }

    /// "total" is a signal word — removed from tokens, captured in signals.
    #[test]
    fn test_total_is_a_signal() {
        let input = vec![
            w("the"), w("total"), w("cost"), w("are"), n(50.0),
        ];
        let out = filter(input);
        assert_eq!(out.tokens, vec![w("cost"), n(50.0)]);
        assert_eq!(out.signals, vec!["total".to_string(), "are".to_string()]);
    }

    /// Numbers and punctuation always pass through untouched.
    #[test]
    fn test_numbers_and_punctuation_pass_through() {
        let input = vec![
            w("find"), w("the"), n(25.0), p('?'),
        ];
        let out = filter(input);
        assert_eq!(out.tokens, vec![n(25.0), p('?')]);
        assert!(out.signals.is_empty());
    }

    /// Multiple signal words are all collected in order.
    #[test]
    fn test_multiple_signals_collected_in_order() {
        let input = vec![
            w("x"), w("is"), n(5.0), w("and"), w("y"), w("are"), n(10.0),
        ];
        let out = filter(input);
        assert_eq!(out.tokens, vec![w("x"), n(5.0), w("y"), n(10.0)]);
        assert_eq!(
            out.signals,
            vec!["is".to_string(), "are".to_string()]
        );
    }

    /// A clean sentence with no noise or signals passes through unchanged.
    #[test]
    fn test_clean_sentence_unchanged() {
        let input = vec![
            w("john"), w("has"), n(5.0), w("apples"),
        ];
        let out = filter(input.clone());
        assert_eq!(out.tokens, input);
        assert!(out.signals.is_empty());
    }

    /// Full pipeline simulation: lexer output → filter.
    #[test]
    fn test_full_pipeline_noise_and_signals() {
        // Simulates lexer output for:
        // "what is the total if john has five apples and mary has three?"
        let input = vec![
            w("what"), w("is"), w("the"), w("total"), w("if"),
            w("john"), w("has"), n(5.0), w("apples"),
            w("and"), w("mary"), w("has"), n(3.0), p('?'),
        ];
        let out = filter(input);
        assert_eq!(
            out.tokens,
            vec![
                w("john"), w("has"), n(5.0), w("apples"),
                w("mary"), w("has"), n(3.0), p('?'),
            ]
        );
        assert_eq!(
            out.signals,
            vec!["is".to_string(), "total".to_string()]
        );
    }
}
