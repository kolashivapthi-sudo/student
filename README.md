# STUDENT

A Rust reimplementation of the classic **STUDENT** program — originally designed by Daniel Bobrow in LISP (1964) as part of his MIT PhD thesis. STUDENT reads algebra word problems written in plain English and solves them.

---

## Quick Demo

```bash
$ cargo run -- "If 2 times a number is 10, what is the number?"

[+] Tokenizing & Filtering...
[+] Translating to equations...
[+] Flattening: _t0 = 2 * VAR_NUMBER
[+] Flattening: _t0 = 10
[✓] Solved: VAR_NUMBER = 5
```

---

## What is STUDENT?

STUDENT is one of the earliest Natural Language Processing (NLP) programs in AI history. It was built to understand and solve algebra word problems like:

> "If the number of customers Tom gets is twice the square of 20% of the number of advertisements he runs, and the number of advertisements he runs is 45, what is the number of customers Tom gets?"

While Joseph Weizenbaum's **ELIZA** (1966) and Bobrow's **STUDENT** (1964) both used pattern matching, they aimed for completely different engineering goals:

| | ELIZA | STUDENT |
|---|---|---|
| Approach | Pattern matching → mirrors input | Pattern matching → computes answer |
| Output | Rephrased version of your input | Solved algebraic answer |
| Intelligence | Simulates understanding | Performs actual math reasoning |

---

## How It Works

The full pipeline:

```
stdin/args → lexer → filter → translator → flattener → solver → formatter → stdout
```

### 1. Lexer — Tokenize
Breaks the raw input string into typed tokens: `Word`, `Number`, `Operator`, `Punctuation` — defined as Rust enums in `types.rs`.

### 2. Filter — Clean the Noise
Strips grammatical filler words that don't contribute to the math. Punctuation like `,` and `.` is handled carefully (following Bobrow's original `|,|` / `|.|` escaping approach in LISP).

### 3. Translator — English → Algebra
Converts phrases into an expression tree (`Expr` enum). The sentence is split into **condition** (LHS — what we know) and **question** (RHS — what to find). Words map to math symbols using pattern matching; prefix notation is used internally.

### 4. Flattener — Normalize to Flat Equations
Takes deeply nested `BinaryOp` expression trees and lifts sub-expressions into fresh intermediate variables (`_t0`, `_t1`, …), producing a flat list of equations where every RHS is a single operation between two leaves.

### 5. Solver — Variable Substitution
Iteratively substitutes known variable values into the equation set until all unknowns are resolved. Includes domain guards that reject out-of-scope inputs (trig, calculus, matrices) with a clear error rather than silently producing wrong output.

### 6. Formatter — Output
Extracts the question variables from the original token stream (by scanning for trigger words like `find`, `what`, `how many`) and displays their solved values.

---

## Project Structure

```
student/
├── src/
│   ├── main.rs         # CLI entry point, question-variable extraction, REPL loop
│   ├── types.rs        # Core AST enums: Token, Operator, Expr, Equation
│   ├── lexer.rs        # Tokenizes raw input into typed Token stream
│   ├── filter.rs       # Noise word removal and phrase normalization
│   ├── translator.rs   # Pattern matching — English phrases → Expr trees
│   ├── flattener.rs    # Lifts nested Expr trees into flat linear equations
│   ├── solver.rs       # Iterative variable substitution engine + domain guards
│   ├── formatter.rs    # Result formatting and stdout output
│   └── error.rs        # Custom SolverError types
├── Cargo.toml
└── Cargo.lock
```

---

## Built With

- **Rust** — recursive enum-based AST (`Expr`, `Token`, `Operator`), pattern matching across pipeline stages, `HashMap`-backed variable substitution in the solver, custom error types via `SolverError`
- No external crates — zero dependencies beyond the standard library

---

## The Ceiling of Rule-Based NLP — and What Comes Next

STUDENT works brilliantly within its boundaries — but those boundaries are real.

Consider this sentence:

> "John has 3 apples and he ate 1 apple, how many apples is he left with?"

STUDENT would fail here. Not because the math is hard (3 − 1 = 2 is trivial), but because of two deeper problems:

**1. Missing world knowledge**
STUDENT only knows a fixed set of keyword templates: "twice", "percent of", "the number of". It has no concept that *ate* means to consume and reduce a quantity. Extending the keyword list helps, but it stays brittle — the real world has too many ways to say the same thing.

**2. Coreference**
"He" refers to John. STUDENT cannot resolve pronouns. It treats every noun as an isolated variable, so it can't track that the subject performing the action is the same person who owns the apples.

### What would actually fix this?

| Approach | What it gives you |
|---|---|
| Expanded keyword rules | More coverage, still brittle |
| Word embeddings (Word2Vec etc.) | "ate" ≈ "consumed" ≈ "used up" — semantic similarity |
| Neural NLP (BERT, LLMs) | Full context, coreference, real-world intent |
| Knowledge graph / ontology | "eating food" → reduces quantity |

A neural network alone isn't enough — what's needed is a model trained on **semantic relationships between language and math operations**. That's the direction modern NLP took: from hand-crafted rules → statistical models → neural embeddings → large language models.

STUDENT is where that journey began. Its architectural ceiling is exactly what motivated 60 years of NLP research.

---

## Background / Inspiration

This repo is part of a study of classic AI programs from the WINGS learning series. The key insight from Bobrow's approach:

- AI handles longer operations by working **outside-in** (like peeling an onion)
- It recurses until it reaches a **base case**
- LISP's prefix notation made it natural to represent equations as nested lists

See [WINGS_LEARNINGS wiki](https://github.com/kolashivapthi-sudo/WINGS_LEARNINGS/wiki) for detailed notes on the concepts behind this implementation.

---

## Getting Started

**Prerequisites:** Rust installed — [rustup.rs](https://rustup.rs)

```bash
git clone https://github.com/kolashivapthi-sudo/student.git
cd student
cargo run
```

Or pass a problem directly:

```bash
cargo run -- "If 2 times a number is 10, what is the number?"
```

---

## References

- Bobrow, D.G. (1964). *Natural Language Input for a Computer Problem Solving System*. MIT PhD Thesis. [PDF](https://dspace.mit.edu/handle/1721.1/5922)
- Weizenbaum, J. (1966). *ELIZA — A Computer Program for the Study of Natural Language Communication Between Man and Machine*. Communications of the ACM.
