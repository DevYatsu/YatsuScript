//! Token stream management for the parser.
//!
//! Wraps the raw token vector produced by the lexer and provides
//! convenience methods for advancing, peeking, and skipping.

use crate::compiler::Loc;
use crate::lexer::Token;

#[derive(Clone, Copy)]
pub struct VarInfo {
    pub idx: usize,
    pub is_mut: bool,
    pub is_global: bool,
    pub first_line: usize,
}

pub struct TokenData<'source> {
    pub token: Token<'source>,
    pub loc: Loc,
}

/// A cursor over the lexed token stream.
///
/// Owns the token vector and current position. The parser
/// holds one of these and delegates navigation to it.
pub struct TokenStream<'source> {
    tokens: Vec<TokenData<'source>>,
    pos: usize,
}

impl<'source> TokenStream<'source> {
    pub fn new(tokens: Vec<TokenData<'source>>) -> Self {
        Self { tokens, pos: 0 }
    }
}
