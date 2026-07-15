//! YatsuScript code formatter.
//!
//! Token-based formatter that normalizes whitespace (no AST needed).

use crate::lexer::Token;

/// Format YatsuScript source code.
pub fn format_source(source: &str) -> String {
    let lexer = <Token as logos::Logos>::lexer(source);
    let tokens: Vec<_> = lexer.flatten().collect();
    format_tokens(&tokens)
}

fn format_tokens(tokens: &[Token<'_>]) -> String {
    let mut output = String::with_capacity(tokens.len() * 4);
    let mut indent: usize = 0;
    let mut line_start = true;

    for (i, token) in tokens.iter().enumerate() {
        match token {
            Token::LBrace => {
                if !line_start { output.push(' '); }
                output.push('{'); output.push('\n');
                indent += 1; line_start = true;
            }
            Token::RBrace => {
                indent = indent.saturating_sub(1);
                if !line_start { output.push('\n'); }
                output.push_str(&"  ".repeat(indent));
                output.push('}'); output.push('\n');
                line_start = true;
            }
            Token::Newline => {
                if !line_start { output.push('\n'); line_start = true; }
            }
            _ => {
                if line_start {
                    output.push_str(&"  ".repeat(indent));
                    line_start = false;
                } else if i > 0 && !matches!(tokens[i-1], Token::Dot)
                    && token != &Token::RParen && token != &Token::RBracket
                    && token != &Token::Comma && token != &Token::Range
                    && token != &Token::Pipe
                {
                    output.push(' ');
                }
                output.push_str(&token_display(token));
            }
        }
    }
    output
}

fn token_display(t: &Token<'_>) -> String {
    match t {
        Token::Fun => "fun".into(),   Token::Ret => "ret".into(),
        Token::If => "if".into(),    Token::Else => "else".into(),
        Token::For => "for".into(),   Token::While => "while".into(),
        Token::In => "in".into(),    Token::And => "and".into(),
        Token::Or => "or".into(),    Token::Nil => "nil".into(),
        Token::Exp => "exp".into(),   Token::Use => "use".into(),
        Token::Super => "super".into(), Token::Move => "move".into(),
        Token::Async => "async".into(), Token::Await => "await".into(),
        Token::Switch => "switch".into(),Token::Break => "break".into(),
        Token::Except => "except".into(),Token::Fail => "fail".into(),
        Token::Error => "error".into(), Token::Continue => "continue".into(),
        Token::Yield => "yield".into(),
        Token::Bool(true) => "true".into(),  Token::Bool(false) => "false".into(),
        Token::Plus => "+".into(), Token::Minus => "-".into(), Token::Mul => "*".into(),
        Token::Div => "/".into(), Token::Mod => "%".into(),
        Token::Eq => "==".into(), Token::Ne => "!=".into(),
        Token::Lt => "<".into(),  Token::Le => "<=".into(),
        Token::Gt => ">".into(),  Token::Ge => ">=".into(),
        Token::Equals => "=".into(),  Token::Not => "!".into(),
        Token::Dot => ".".into(),  Token::Range => "..".into(), Token::Arrow => "->".into(),
        Token::Pipe => "|".into(),  Token::Colon => ":".into(), Token::Comma => ",".into(),
        Token::Semicolon => ";".into(),
        Token::PlusEq => "+=".into(), Token::MinusEq => "-=".into(), Token::MulEq => "*=".into(),
        Token::DivEq => "/=".into(), Token::ModEq => "%=".into(),
        Token::LParen => "(".into(), Token::RParen => ")".into(),
        Token::LBrace => "{".into(), Token::RBrace => "}".into(),
        Token::LBracket => "[".into(), Token::RBracket => "]".into(),
        Token::Number(n) => n.to_string(),
        Token::String(s) => format!("\"{}\"", s),
        Token::Template(s) => format!("`{}`", s),
        Token::Identifier(s) => s.to_string(),
        Token::LineComment => String::new(),
        Token::Newline => "\n".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_simple() {
        let input = "fun add(a,b){ret a+b}";
        let out = format_source(input);
        assert!(out.contains("fun add(a, b) {"));
        assert!(out.contains("  ret a + b"));
    }
}
