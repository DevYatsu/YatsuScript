//! Runtime and compile-time error types for the Pi interpreter.
//!
//! All errors carry a human-readable message and a `(line, col)` location that
//! points back into the original source text, allowing [`crate::print_error`]
//! to render a helpful caret-annotated diagnostic.

use crate::lexer::LexingError;
use thiserror::Error;

/// Every error that can be produced while compiling or running a Pi program.
///
/// Variants are ordered roughly by the pipeline stage that produces them:
/// lexing → parsing → runtime.
#[derive(Error, Debug, Clone)]
pub enum JitError {
    /// A character or token sequence that the lexer cannot recognise.
    ///
    /// Contains the underlying [`LexingError`] and the `(line, col)` of the
    /// offending character.
    #[error("Lexing error at {1}:{2}:  {0:?}")]
    Lexing(LexingError, usize, usize),

    /// A syntactically invalid construct encountered during parsing or
    /// bytecode generation.
    ///
    /// The contained string is a human-readable description of what was
    /// expected vs. what was found.
    #[error("Parsing error at {1}:{2}:  {0}")]
    Parsing(String, usize, usize),

    /// An error that occurs at runtime (e.g. type mismatch, index out of
    /// bounds, unknown native function).
    #[error("Runtime error at {1}:{2}:  {0}")]
    Runtime(String, usize, usize),

    /// A variable name was referenced that had never been declared.
    #[error("Unknown variable at {1}:{2}:  {0}")]
    UnknownVariable(String, usize, usize),

    /// An attempt was made to re-assign a variable declared with `let`
    /// (immutable), or to re-declare it in the same scope.
    ///
    /// The fourth field is the line on which the variable was *originally*
    /// defined, to aid diagnosis.
    #[error("Redefinition of immutable variable at {1}:{2}: '{0}' was already defined on line {3}")]
    RedefinitionOfImmutableVariable(String, usize, usize, usize),
}

impl JitError {
    /// Return the `(line, column)` source location where this error occurred.
    ///
    /// Lines and columns are 1-based.  Returns `(0, 0)` for errors that do
    /// not have a meaningful location.
    pub fn location(&self) -> (usize, usize) {
        match self {
            JitError::Lexing(_, line, col) => (*line, *col),
            JitError::Parsing(_, line, col) => (*line, *col),
            JitError::Runtime(_, line, col) => (*line, *col),
            JitError::UnknownVariable(_, line, col) => (*line, *col),
            JitError::RedefinitionOfImmutableVariable(_, line, col, _) => (*line, *col),
        }
    }
}
