use std::fmt;

/// All errors the solver pipeline can produce.
#[derive(Debug, Clone, PartialEq)]
pub enum SolverError {
    /// Attempted to divide by zero during evaluation.
    DivisionByZero,

    /// Not enough equations to solve for all unknowns.
    InsufficientInformation,

    /// Problem requires solving a polynomial of degree > 1 (quadratic, cubic, etc.).
    /// Stores the degree detected, e.g. 2 for quadratic.
    UnsupportedDegree(u32),

    /// Problem contains trigonometry, calculus, or other out-of-scope domains.
    /// Stores the offending keyword found, e.g. "sin", "integral".
    UnsupportedDomain(String),

    /// Input sentence could not be parsed into equations.
    ParseError(String),
}

impl fmt::Display for SolverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SolverError::DivisionByZero => {
                write!(f, "Division by zero is not possible.")
            }
            SolverError::InsufficientInformation => {
                write!(f, "Not enough information to solve the problem.")
            }
            SolverError::UnsupportedDegree(degree) => {
                write!(
                    f,
                    "This problem requires degree-{} algebra, which is beyond high school scope.",
                    degree
                )
            }
            SolverError::UnsupportedDomain(domain) => {
                write!(
                    f,
                    "Unsupported topic detected: '{}'. Only basic algebra is supported.",
                    domain
                )
            }
            SolverError::ParseError(msg) => {
                write!(f, "Could not understand the problem: {}", msg)
            }
        }
    }
}

impl std::error::Error for SolverError {}
