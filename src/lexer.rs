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
}
