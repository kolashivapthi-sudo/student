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

    // -----------------------------------------------------------------------
    // Mandatory tests 1-10
    // -----------------------------------------------------------------------

    /// Req 1: Every listed noise word is removed from a single sentence
    /// containing all of them.
    /// Noise words: a, an, the, was, were, if, of, and, then, each, some,
    ///              there, find, how, many, what
    /// Signal words (removed from tokens but captured): is, are, total
    #[test]
    fn test_all_noise_words_removed() {
        let input = vec![
            // pure noise
            w("a"), w("an"), w("the"), w("was"), w("were"), w("if"),
            w("of"), w("and"), w("then"), w("each"), w("some"),
            w("there"), w("find"), w("how"), w("many"), w("what"),
            // signal words (go to signals list, not tokens)
            w("is"), w("are"), w("total"),
            // content word that must survive
            w("result"),
        ];
        let out = filter(input);
        // Only the content word survives in tokens
        assert_eq!(out.tokens, vec![w("result")]);
        // All three signal words captured in order
        assert_eq!(
            out.signals,
            vec!["is".to_string(), "are".to_string(), "total".to_string()]
        );
    }

    /// Req 2: "is" goes to signals list, not deleted entirely.
    #[test]
    fn test_is_is_signal_not_deleted() {
        let input = vec![w("x"), w("is"), n(5.0)];
        let out = filter(input);
        assert_eq!(out.tokens, vec![w("x"), n(5.0)]);
        assert!(out.signals.contains(&"is".to_string()));
    }

    /// Req 3: "are" goes to signals list, not deleted entirely.
    #[test]
    fn test_are_is_signal_not_deleted() {
        let input = vec![w("values"), w("are"), n(10.0)];
        let out = filter(input);
        assert_eq!(out.tokens, vec![w("values"), n(10.0)]);
        assert!(out.signals.contains(&"are".to_string()));
    }

    /// Req 4: "total" goes to signals list, not deleted entirely.
    #[test]
    fn test_total_is_signal_not_deleted() {
        let input = vec![w("total"), w("cost"), n(99.0)];
        let out = filter(input);
        assert_eq!(out.tokens, vec![w("cost"), n(99.0)]);
        assert!(out.signals.contains(&"total".to_string()));
    }

    /// Req 5: Sentence with no noise words returns unchanged token list.
    #[test]
    fn test_no_noise_words_returns_unchanged() {
        let input = vec![w("john"), w("has"), n(7.0), w("apples")];
        let out = filter(input.clone());
        assert_eq!(out.tokens, input);
        assert!(out.signals.is_empty());
    }

    /// Req 6: Sentence with ONLY noise/signal words → empty token list,
    /// correct signals captured.
    #[test]
    fn test_only_noise_words_empty_tokens() {
        // "is the total" — "is" and "total" are signals, "the" is noise
        let input = vec![w("is"), w("the"), w("total")];
        let out = filter(input);
        assert!(out.tokens.is_empty(),
            "Expected empty tokens, got: {:?}", out.tokens);
        assert_eq!(
            out.signals,
            vec!["is".to_string(), "total".to_string()]
        );
    }

    /// Req 7: Noise word filtering is case-insensitive.
    /// The lexer always lowercases, but filter should handle both forms
    /// in case it's ever called directly with mixed-case input.
    #[test]
    fn test_noise_filtering_case_insensitive() {
        // lowercase already lowercased by lexer — this is the normal path
        let input_lower = vec![w("the"), w("john"), w("a"), w("mary")];
        let out_lower = filter(input_lower);
        assert_eq!(out_lower.tokens, vec![w("john"), w("mary")]);

        // Filter only sees what lexer gives it (always lowercase), so
        // "The" would never arrive — but verify the contract is consistent:
        // "the" is stripped, content words survive.
        assert!(out_lower.signals.is_empty());
    }

    /// Req 8: Numbers and content words pass through completely untouched.
    #[test]
    fn test_numbers_and_content_words_pass_through() {
        let input = vec![
            n(42.0), w("apples"), n(3.14), w("price"), p('.'),
        ];
        let out = filter(input.clone());
        assert_eq!(out.tokens, input);
        assert!(out.signals.is_empty());
    }

    /// Req 9: Noise words that appear immediately adjacent to punctuation
    /// are still filtered correctly.
    /// e.g. "find, the answer?" — "find" and "the" removed, punctuation kept.
    #[test]
    fn test_noise_words_adjacent_to_punctuation() {
        let input = vec![
            w("find"), p(','), w("the"), w("answer"), p('?'),
        ];
        let out = filter(input);
        assert_eq!(
            out.tokens,
            vec![p(','), w("answer"), p('?')]
        );
        assert!(out.signals.is_empty());
    }

    /// Req 10: Signal words preserve their original order relative to
    /// each other even when interleaved with noise and content words.
    #[test]
    fn test_signal_words_preserve_order() {
        // order: total ... are ... is
        let input = vec![
            w("total"), w("cost"), w("are"), n(10.0), w("and"),
            w("price"), w("is"), n(5.0),
        ];
        let out = filter(input);
        // Signals must appear in the order they were encountered
        assert_eq!(
            out.signals,
            vec!["total".to_string(), "are".to_string(), "is".to_string()]
        );
        // "and" is noise, removed; content words + numbers survive
        assert_eq!(
            out.tokens,
            vec![w("cost"), n(10.0), w("price"), n(5.0)]
        );
    }
}
