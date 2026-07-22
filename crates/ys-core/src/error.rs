use crate::lexer::LexingError;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ErrorLoc {
    pub line: usize,
    pub col: usize,
}

impl ErrorLoc {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

impl std::fmt::Display for ErrorLoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

/// Every error that can be produced while compiling or running a ysc program.
#[derive(Error, Debug, Clone)]
pub enum JitError {
    #[error("Lexing error at {loc}: {err:?}")]
    Lexing { err: LexingError, loc: ErrorLoc },

    #[error("Parsing error at {loc}: {msg}")]
    Parsing { msg: String, loc: ErrorLoc },

    #[error("Runtime error at {loc}: {msg}")]
    Runtime { msg: String, loc: ErrorLoc },

    #[error("Unknown variable at {loc}: {msg}")]
    UnknownVariable { msg: String, loc: ErrorLoc },

    #[error(
        "Redefinition of immutable variable at {loc}: '{msg}' was already defined on line {orig_line}"
    )]
    RedefinitionOfImmutableVariable {
        msg: String,
        loc: ErrorLoc,
        orig_line: usize,
    },
}

impl JitError {
    pub fn location(&self) -> (usize, usize) {
        let loc = match self {
            JitError::Lexing { loc, .. } => loc,
            JitError::Parsing { loc, .. } => loc,
            JitError::Runtime { loc, .. } => loc,
            JitError::UnknownVariable { loc, .. } => loc,
            JitError::RedefinitionOfImmutableVariable { loc, .. } => loc,
        };
        (loc.line, loc.col)
    }

    pub fn runtime(msg: impl Into<String>, pos: (usize, usize)) -> Self {
        Self::Runtime {
            msg: msg.into(),
            loc: ErrorLoc::new(pos.0, pos.1),
        }
    }

    pub fn parsing(msg: impl Into<String>, pos: (usize, usize)) -> Self {
        Self::Parsing {
            msg: msg.into(),
            loc: ErrorLoc::new(pos.0, pos.1),
        }
    }

    pub fn unknown_variable(msg: impl Into<String>, pos: (usize, usize)) -> Self {
        Self::UnknownVariable {
            msg: msg.into(),
            loc: ErrorLoc::new(pos.0, pos.1),
        }
    }
}
