use crate::types::Token;

/// Maps English number-words to their numeric values.
/// Covers zero through twenty, plus common tens (thirty … ninety)
/// and hundred/thousand for high-school-level problems.
fn word_to_number(word: &str) -> Option<f64> {
    match word {
        "zero"      => Some(0.0),
        "one"       => Some(1.0),
        "two"       => Some(2.0),
        "three"     => Some(3.0),
        "four"      => Some(4.0),
        "five"      => Some(5.0),
        "six"       => Some(6.0),
        "seven"     => Some(7.0),
        "eight"     => Some(8.0),
        "nine"      => Some(9.0),
        "ten"       => Some(10.0),
        "eleven"    => Some(11.0),
        "twelve"    => Some(12.0),
        "thirteen"  => Some(13.0),
        "fourteen"  => Some(14.0),
        "fifteen"   => Some(15.0),
        "sixteen"   => Some(16.0),
        "seventeen" => Some(17.0),
        "eighteen"  => Some(18.0),
        "nineteen"  => Some(19.0),
        "twenty"    => Some(20.0),
        "thirty"    => Some(30.0),
        "forty"     => Some(40.0),
        "fifty"     => Some(50.0),
        "sixty"     => Some(60.0),
        "seventy"   => Some(70.0),
        "eighty"    => Some(80.0),
        "ninety"    => Some(90.0),
        "hundred"   => Some(100.0),
        "thousand"  => Some(1000.0),
        _           => None,
    }
}

/// Returns true if the character is recognised punctuation.
fn is_punctuation(c: char) -> bool {
    matches!(c, '.' | ',' | '?' | '!' | ';' | ':')
}

/// Tokenizes a raw English sentence into a `Vec<Token>`.
///
/// Steps:
/// 1. Lowercase the entire input (case-insensitive handling).
/// 2. Walk character-by-character, collecting:
///    - digit runs  → `Token::Number`
///    - alpha runs  → check word-to-number map first, else `Token::Word`
///    - punctuation → `Token::Punctuation`
///    - whitespace  → separator, ignored
pub fn tokenize(input: &str) -> Vec<Token> {
    let input = input.to_lowercase();
    let chars: Vec<char> = input.chars().collect();
    let mut tokens: Vec<Token> = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        // --- whitespace: skip ---
        if c.is_whitespace() {
            i += 1;
            continue;
        }

        // --- digit run (handles integers and decimals like 3.5) ---
        if c.is_ascii_digit() {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            let num_str: String = chars[start..i].iter().collect();
            if let Ok(n) = num_str.parse::<f64>() {
                tokens.push(Token::Number(n));
            }
            continue;
        }

        // --- alphabetic run ---
        if c.is_alphabetic() {
            let start = i;
            while i < chars.len() && chars[i].is_alphabetic() {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();

            if let Some(n) = word_to_number(&word) {
                tokens.push(Token::Number(n));
            } else {
                tokens.push(Token::Word(word));
            }
            continue;
        }

        // --- punctuation ---
        if is_punctuation(c) {
            tokens.push(Token::Punctuation(c));
            i += 1;
            continue;
        }

        // --- anything else (e.g. hyphens, apostrophes): skip ---
        i += 1;
    }

    tokens
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Token;

    /// Simple sentence — all plain words, no numbers.
    #[test]
    fn test_simple_sentence() {
        let tokens = tokenize("John has apples");
        assert_eq!(
            tokens,
            vec![
                Token::Word("john".into()),
                Token::Word("has".into()),
                Token::Word("apples".into()),
            ]
        );
    }

    /// Sentence containing digit literals.
    #[test]
    fn test_sentence_with_digits() {
        let tokens = tokenize("John has 25 apples and 3 oranges");
        assert_eq!(
            tokens,
            vec![
                Token::Word("john".into()),
                Token::Word("has".into()),
                Token::Number(25.0),
                Token::Word("apples".into()),
                Token::Word("and".into()),
                Token::Number(3.0),
                Token::Word("oranges".into()),
            ]
        );
    }

    /// Sentence containing English number-words.
    #[test]
    fn test_sentence_with_number_words() {
        let tokens = tokenize("Mary has twelve cookies and five brownies");
        assert_eq!(
            tokens,
            vec![
                Token::Word("mary".into()),
                Token::Word("has".into()),
                Token::Number(12.0),
                Token::Word("cookies".into()),
                Token::Word("and".into()),
                Token::Number(5.0),
                Token::Word("brownies".into()),
            ]
        );
    }

    /// Sentence ending with punctuation.
    #[test]
    fn test_sentence_with_punctuation() {
        let tokens = tokenize("How many apples does John have?");
        assert_eq!(
            tokens,
            vec![
                Token::Word("how".into()),
                Token::Word("many".into()),
                Token::Word("apples".into()),
                Token::Word("does".into()),
                Token::Word("john".into()),
                Token::Word("have".into()),
                Token::Punctuation('?'),
            ]
        );
    }

    /// Mixed: digits, number-words, punctuation, and mixed-case input.
    #[test]
    fn test_mixed_and_case_insensitive() {
        let tokens = tokenize("Alice has 3 more than TWENTY apples, right?");
        assert_eq!(
            tokens,
            vec![
                Token::Word("alice".into()),
                Token::Word("has".into()),
                Token::Number(3.0),
                Token::Word("more".into()),
                Token::Word("than".into()),
                Token::Number(20.0),
                Token::Word("apples".into()),
                Token::Punctuation(','),
                Token::Word("right".into()),
                Token::Punctuation('?'),
            ]
        );
    }

    /// Decimal number handling.
    #[test]
    fn test_decimal_number() {
        let tokens = tokenize("The price is 3.5 dollars");
        assert_eq!(
            tokens,
            vec![
                Token::Word("the".into()),
                Token::Word("price".into()),
                Token::Word("is".into()),
                Token::Number(3.5),
                Token::Word("dollars".into()),
            ]
        );
    }

    // -----------------------------------------------------------------------
    // Mandatory tests 1-10
    // -----------------------------------------------------------------------

    /// Req 1: Simple sentence tokenizes into correct word tokens.
    /// "The sum is 10" → [the, sum, is, 10]
    #[test]
    fn test_simple_sentence_the_sum_is_10() {
        let tokens = tokenize("The sum is 10");
        assert_eq!(
            tokens,
            vec![
                Token::Word("the".into()),
                Token::Word("sum".into()),
                Token::Word("is".into()),
                Token::Number(10.0),
            ]
        );
    }

    /// Req 2: Digit numbers 20 and 100 parse correctly.
    #[test]
    fn test_digit_numbers_20_and_100() {
        let tokens = tokenize("20 plus 100");
        assert_eq!(
            tokens,
            vec![
                Token::Number(20.0),
                Token::Word("plus".into()),
                Token::Number(100.0),
            ]
        );
    }

    /// Req 3: Number-words "two", "four", "twenty" parse to their numeric values.
    #[test]
    fn test_number_words_two_four_twenty() {
        let tokens = tokenize("two plus four equals twenty");
        assert_eq!(
            tokens,
            vec![
                Token::Number(2.0),
                Token::Word("plus".into()),
                Token::Number(4.0),
                Token::Word("equals".into()),
                Token::Number(20.0),
            ]
        );
    }

    /// Req 4: Mixed digit literals and number-words in one sentence.
    #[test]
    fn test_mixed_digits_and_number_words() {
        let tokens = tokenize("3 times twelve is thirty six");
        assert_eq!(
            tokens,
            vec![
                Token::Number(3.0),
                Token::Word("times".into()),
                Token::Number(12.0),
                Token::Word("is".into()),
                Token::Number(30.0),
                Token::Number(6.0),
            ]
        );
    }

    /// Req 5: Period, comma, and question mark are each tokenized as Punctuation.
    #[test]
    fn test_all_three_punctuation_types() {
        let tokens = tokenize("Wait, really. Are you sure?");
        // period, comma, and question mark all appear
        assert!(tokens.contains(&Token::Punctuation('.')));
        assert!(tokens.contains(&Token::Punctuation(',')));
        assert!(tokens.contains(&Token::Punctuation('?')));
    }

    /// Req 6: Case-insensitivity — "The" and "the" produce identical tokens.
    #[test]
    fn test_case_insensitivity_the() {
        let upper = tokenize("The");
        let lower = tokenize("the");
        assert_eq!(upper, lower);
        assert_eq!(upper, vec![Token::Word("the".into())]);
    }

    /// Req 7: Multiple spaces and extra whitespace do not break tokenization.
    #[test]
    fn test_extra_whitespace_ignored() {
        let tokens = tokenize("john   has    5   apples");
        assert_eq!(
            tokens,
            vec![
                Token::Word("john".into()),
                Token::Word("has".into()),
                Token::Number(5.0),
                Token::Word("apples".into()),
            ]
        );
    }

    /// Req 8: Empty string returns an empty token list without panicking.
    #[test]
    fn test_empty_string_returns_empty_vec() {
        let tokens = tokenize("");
        assert!(tokens.is_empty());
    }

    /// Req 9: Sentence with only numbers and no words.
    #[test]
    fn test_only_numbers_no_words() {
        let tokens = tokenize("10 20 30");
        assert_eq!(
            tokens,
            vec![
                Token::Number(10.0),
                Token::Number(20.0),
                Token::Number(30.0),
            ]
        );
    }

    /// Req 10: Hyphenated word "twenty-five" — hyphen is skipped,
    /// producing two separate tokens: Number(20) and Number(5).
    /// This is the defined reasonable behaviour: hyphens are stripped,
    /// each side is tokenized independently.
    #[test]
    fn test_hyphenated_number_word() {
        let tokens = tokenize("twenty-five");
        // hyphen stripped → "twenty" = 20.0, "five" = 5.0
        assert_eq!(
            tokens,
            vec![
                Token::Number(20.0),
                Token::Number(5.0),
            ]
        );
    }

    /// Req 10 (words): Hyphenated plain word "well-known" splits into two Word tokens.
    #[test]
    fn test_hyphenated_plain_word() {
        let tokens = tokenize("well-known fact");
        assert_eq!(
            tokens,
            vec![
                Token::Word("well".into()),
                Token::Word("known".into()),
                Token::Word("fact".into()),
            ]
        );
    }
}
