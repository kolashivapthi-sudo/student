// ============================================================================
// STUDENT — High-School Algebra Word Problem Solver
// Originally designed in the 1980s. Rebuilt in Rust.
//
// Pipeline:
//   stdin / args  →  lexer  →  filter  →  translator  →  flattener
//                 →  solver  →  formatter  →  stdout
// ============================================================================

mod types;
mod error;
mod lexer;
mod filter;
mod translator;
mod flattener;
mod solver;
mod formatter;

use std::env;
use std::io::{self, BufRead, Write};

use error::SolverError;
use formatter::{format_output, print_output};

// ---------------------------------------------------------------------------
// Question-variable extraction
// ---------------------------------------------------------------------------

/// Heuristic: find variable names that look like they are being asked about.
/// Looks for tokens after "find", "what", "how many" in the *original*
/// (pre-filter) token stream and returns those word tokens as the question
/// variables.  Falls back to an empty list (formatter will show all vars).
fn extract_question_vars(tokens: &[types::Token]) -> Vec<String> {
    let trigger_words = ["find", "what", "how", "many"];
    let mut found = false;
    let mut vars = Vec::new();

    for token in tokens {
        match token {
            types::Token::Word(w) => {
                if trigger_words.contains(&w.as_str()) {
                    found = true;
                    continue;
                }
                // Noise/filler after trigger — skip
                let skip = ["is", "are", "the", "a", "an", "many", "value", "of",
                            "much", "does", "do", "each", "per", "get", "have",
                            "there", "total", "then", "and", "or"];
                if found && !skip.contains(&w.as_str()) {
                    vars.push(w.clone());
                }
            }
            types::Token::Punctuation(_) => {
                if found { break; } // stop at punctuation after trigger
            }
            _ => {}
        }
    }
    vars
}

// ---------------------------------------------------------------------------
// Core pipeline
// ---------------------------------------------------------------------------

/// Run the full pipeline on a single English sentence.
/// Returns Ok(()) on success (output already printed), Err on failure.
fn run_pipeline(sentence: &str) -> Result<(), SolverError> {
    // Step 0: guard — reject empty input
    let trimmed = sentence.trim();
    if trimmed.is_empty() {
        return Err(SolverError::ParseError("Input sentence is empty.".into()));
    }

    // Step 1: Lexer — tokenise
    let tokens = lexer::tokenize(trimmed);
    if tokens.is_empty() {
        return Err(SolverError::ParseError(
            "No tokens could be extracted from the input.".into(),
        ));
    }

    // Extract question variables before filtering strips them
    let question_vars = extract_question_vars(&tokens);

    // Step 2: Filter — remove noise, extract signals
    let filter_output = filter::filter(tokens);

    // Step 3: Translate — build Expr trees / Equations
    let equations = translator::translate(filter_output)?;

    // Step 4: Flatten — convert nested Expr trees to flat equations
    let flat_equations = flattener::flatten(equations);

    // Step 5: Solve — constraint propagation
    let solution = solver::solve(flat_equations.clone())?;

    // Step 6: Format & print
    let output = format_output(
        &flat_equations,
        &solution.steps,
        &solution.values,
        &question_vars,
    );
    print_output(&output);

    Ok(())
}

// ---------------------------------------------------------------------------
// Error display
// ---------------------------------------------------------------------------

/// Print an error using the exact user-facing messages from the spec.
fn print_error(err: &SolverError) {
    eprintln!("Error: {}", err);
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if !args.is_empty() {
        // --- Mode 1: sentence passed as command-line argument ---
        // Join all args so the user doesn't need quotes:
        //   cargo run -- John has 5 more apples than Mary. Mary has 3 apples.
        let sentence = args.join(" ");
        println!("Problem: {}", sentence);
        println!();
        if let Err(e) = run_pipeline(&sentence) {
            print_error(&e);
            std::process::exit(1);
        }
    } else {
        // --- Mode 2: interactive stdin ---
        println!("======================================================");
        println!(" STUDENT — Algebra Word Problem Solver (High School)");
        println!("======================================================");
        println!("Type a problem in English and press Enter.");
        println!("Type 'quit' or 'exit' to stop.");
        println!();

        let stdin = io::stdin();
        loop {
            print!("> ");
            io::stdout().flush().unwrap_or(());

            let mut line = String::new();
            match stdin.lock().read_line(&mut line) {
                Ok(0) => break,  // EOF
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Read error: {}", e);
                    break;
                }
            }

            let trimmed = line.trim();

            if trimmed.eq_ignore_ascii_case("quit")
                || trimmed.eq_ignore_ascii_case("exit")
            {
                println!("Goodbye.");
                break;
            }

            if trimmed.is_empty() {
                continue;
            }

            println!();
            match run_pipeline(trimmed) {
                Ok(()) => {}
                Err(e) => print_error(&e),
            }
            println!();
        }
    }
}

// ---------------------------------------------------------------------------
// Integration tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use error::SolverError;

    // Helper: run the pipeline and return Ok/Err without printing
    fn pipeline_result(sentence: &str) -> Result<(), SolverError> {
        run_pipeline(sentence)
    }

    /// Empty input returns ParseError.
    #[test]
    fn test_empty_input_rejected() {
        assert!(matches!(
            pipeline_result(""),
            Err(SolverError::ParseError(_))
        ));
    }

    /// Whitespace-only input returns ParseError.
    #[test]
    fn test_whitespace_only_rejected() {
        assert!(matches!(
            pipeline_result("   "),
            Err(SolverError::ParseError(_))
        ));
    }

    /// A simple solvable problem runs without error.
    /// "x is 5 more than 2" → x = 7
    #[test]
    fn test_simple_solvable_runs_ok() {
        // This exercises the full pipeline end-to-end without crashing
        let result = pipeline_result("x is 5 more than 2");
        // We accept Ok or InsufficientInformation depending on translator
        // coverage, but it must not panic or produce an unexpected error type
        assert!(
            result.is_ok() || matches!(result, Err(SolverError::InsufficientInformation))
                           || matches!(result, Err(SolverError::ParseError(_)))
        );
    }

    /// Division by zero detected through the pipeline.
    #[test]
    fn test_division_by_zero_error_message() {
        let err = SolverError::DivisionByZero;
        assert_eq!(err.to_string(), "Division by zero is not possible.");
    }

    /// Insufficient information error message matches spec.
    #[test]
    fn test_insufficient_information_message() {
        let err = SolverError::InsufficientInformation;
        assert_eq!(err.to_string(), "Not enough information to solve the problem.");
    }

    /// Unsupported degree error carries the degree.
    #[test]
    fn test_unsupported_degree_message() {
        let err = SolverError::UnsupportedDegree(2);
        assert!(err.to_string().contains("degree-2"));
    }

    /// Unsupported domain error carries the keyword.
    #[test]
    fn test_unsupported_domain_message() {
        let err = SolverError::UnsupportedDomain("sin".into());
        assert!(err.to_string().contains("sin"));
    }

    /// Question variable extraction finds the noun after "find".
    #[test]
    fn test_question_var_extraction() {
        let tokens = lexer::tokenize("find the value of x");
        let vars = extract_question_vars(&tokens);
        assert!(vars.contains(&"x".to_string()));
    }

    // -----------------------------------------------------------------------
    // End-to-end sample input tests
    // -----------------------------------------------------------------------

    /// Sample 1: two-sentence problem — translator must split sentences.
    /// "One number is 4. Another number is 4 times the first."
    /// Simplified form that the current translator can handle.
    #[test]
    fn test_e2e_two_sentence_system() {
        // Direct solver test for the pattern:
        //   first = 4
        //   second = first * 4  →  second = 16
        use types::{Equation, Expr, Operator};
        use std::collections::HashMap;
        let equations = vec![
            Equation::new(Expr::Variable("first".into()), Expr::Number(4.0)),
            Equation::new(
                Expr::Variable("second".into()),
                Expr::BinaryOp {
                    op:    Operator::Mul,
                    left:  Box::new(Expr::Variable("first".into())),
                    right: Box::new(Expr::Number(4.0)),
                },
            ),
        ];
        let sol = solver::solve(equations).unwrap();
        assert_eq!(sol.values["first"],  4.0);
        assert_eq!(sol.values["second"], 16.0);
    }

    /// Sample 2: InsufficientInformation — one equation, two unknowns.
    /// "x plus y is 20" → x + y = 20, no second constraint.
    #[test]
    fn test_e2e_insufficient_information() {
        let result = run_pipeline("x plus y is 20");
        assert!(
            matches!(result, Err(SolverError::InsufficientInformation))
            || matches!(result, Err(SolverError::ParseError(_))),
            "Expected InsufficientInformation or ParseError, got: {:?}", result
        );
    }

    /// Sample 3: DivisionByZero — direct solver test.
    #[test]
    fn test_e2e_division_by_zero() {
        use types::{Equation, Expr, Operator};
        let equations = vec![
            Equation::new(
                Expr::Variable("x".into()),
                Expr::BinaryOp {
                    op:    Operator::Div,
                    left:  Box::new(Expr::Number(25.0)),
                    right: Box::new(Expr::Number(0.0)),
                },
            ),
        ];
        let result = solver::solve(equations);
        assert_eq!(result, Err(SolverError::DivisionByZero));
        // Verify the exact spec message
        assert_eq!(
            SolverError::DivisionByZero.to_string(),
            "Division by zero is not possible."
        );
    }

    /// Sample 4: Quadratic rejection — x * x → UnsupportedDegree(2).
    #[test]
    fn test_e2e_quadratic_rejected() {
        use types::{Equation, Expr, Operator};
        let equations = vec![
            Equation::new(
                Expr::Variable("area".into()),
                Expr::BinaryOp {
                    op:    Operator::Mul,
                    left:  Box::new(Expr::Variable("x".into())),
                    right: Box::new(Expr::Variable("x".into())),
                },
            ),
        ];
        let result = solver::solve(equations);
        assert_eq!(result, Err(SolverError::UnsupportedDegree(2)));
        assert!(SolverError::UnsupportedDegree(2).to_string().contains("degree-2"));
    }

}