/// Output tokens produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(String),
    Number(f64),
    Operator(String),   // e.g. "more than", "times", "divided by"
    Punctuation(char),  // e.g. '.', ',', '?'
}

/// Mathematical operators supported in expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}

/// An algebraic expression tree node.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A literal numeric value, e.g. 5.0
    Number(f64),
    /// A named unknown variable, e.g. "x" or "john"
    Variable(String),
    /// A binary operation combining two sub-expressions
    BinaryOp {
        op: Operator,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

/// A single algebraic equation: lhs = rhs
#[derive(Debug, Clone, PartialEq)]
pub struct Equation {
    pub lhs: Expr,
    pub rhs: Expr,
}

impl Equation {
    pub fn new(lhs: Expr, rhs: Expr) -> Self {
        Equation { lhs, rhs }
    }
}
