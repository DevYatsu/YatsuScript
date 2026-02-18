use crate::lexer::LexingError;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum JitError {
    #[error("Lexing error at {1}:{2}:  {0:?}")]
    Lexing(LexingError, usize, usize),
    #[error("Parsing error at {1}:{2}:  {0}")]
    Parsing(String, usize, usize),
    #[error("Runtime error at {1}:{2}:  {0}")]
    Runtime(String, usize, usize),
    #[error("Unknown variable at {1}:{2}:  {0}")]
    UnknownVariable(String, usize, usize),
    #[error("Redefinition of immutable variable at {1}:{2}: '{0}' was already defined on line {3}")]
    RedefinitionOfImmutableVariable(String, usize, usize, usize),
}

impl JitError {
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
