use crate::error::JitError;
use crate::lexer::Token;
use logos::Logos;
use std::fs;
use std::path::Path;

pub fn format_file(path: &Path) -> Result<(), JitError> {
    let content = fs::read_to_string(path).map_err(|e| {
        JitError::Runtime(format!("Failed to read file: {}", e), 0, 0)
    })?;

    let formatted = format_source(&content)?;

    if content != formatted {
        fs::write(path, formatted).map_err(|e| {
            JitError::Runtime(format!("Failed to write file: {}", e), 0, 0)
        })?;
        println!("Formatted {}", path.display());
    }

    Ok(())
}

pub fn format_source(source: &str) -> Result<String, JitError> {
    let mut lexer = Token::lexer(source);
    let mut output = String::new();
    let mut indent_level: usize = 0;
    let mut at_start_of_line = true;
    let mut need_space = false;

    while let Some(token_result) = lexer.next() {
        let token = token_result.map_err(|e| {
            JitError::Lexing(e, 0, 0) // We don't have exact line/col here easily
        })?;

        match token {
            Token::Newline => {
                output.push('\n');
                at_start_of_line = true;
                need_space = false;
                continue;
            }
            Token::RBrace => {
                indent_level = indent_level.saturating_sub(1);
            }
            _ => {}
        }

        if at_start_of_line {
            output.push_str(&"    ".repeat(indent_level));
            at_start_of_line = false;
            need_space = false;
        } else if need_space {
            match token {
                Token::Comma | Token::Colon | Token::LParen | Token::RParen | Token::LBracket | Token::RBracket | Token::Dot => {}
                _ => output.push(' '),
            }
        }

        match token {
            Token::LBrace => {
                output.push('{');
                indent_level += 1;
                need_space = true;
            }
            Token::RBrace => {
                output.push('}');
                need_space = true;
            }
            Token::LParen => {
                output.push('(');
                need_space = false;
            }
            Token::RParen => {
                output.push(')');
                need_space = true;
            }
            Token::LBracket => {
                output.push('[');
                need_space = false;
            }
            Token::RBracket => {
                output.push(']');
                need_space = true;
            }
            Token::Comma => {
                output.push(',');
                need_space = true;
            }
            Token::Colon => {
                output.push(':');
                need_space = true;
            }
            Token::Dot => {
                output.push('.');
                need_space = false;
            }
            Token::Plus | Token::Minus | Token::Mul | Token::Div | Token::Eq | Token::Ne | Token::Lt | Token::Le | Token::Gt | Token::Ge => {
                if !output.ends_with(' ') {
                    output.push(' ');
                }
                output.push_str(lexer.slice());
                output.push(' ');
                need_space = false;
            }
            Token::Identifier(s) => {
                output.push_str(s);
                need_space = true;
            }
            Token::String(s) => {
                output.push('"');
                output.push_str(s);
                output.push('"');
                need_space = true;
            }
            Token::Number(_) | Token::Bool(_) => {
                output.push_str(lexer.slice());
                need_space = true;
            }
            Token::MutableVar => {
                output.push_str("mut");
                need_space = true;
            }
            Token::ImmutableVar => {
                output.push_str("let");
                need_space = true;
            }
            Token::Fn => {
                output.push_str("fn");
                need_space = true;
            }
            Token::Return => {
                output.push_str("return");
                need_space = true;
            }
            Token::If => {
                output.push_str("if");
                need_space = true;
            }
            Token::Else => {
                output.push_str("else");
                need_space = true;
            }
            Token::Spawn => {
                output.push_str("spawn");
                need_space = true;
            }
            Token::For => {
                output.push_str("for");
                need_space = true;
            }
            Token::While => {
                output.push_str("while");
                need_space = true;
            }
            Token::In => {
                output.push_str("in");
                need_space = true;
            }
            Token::Range => {
                output.push_str("..");
                need_space = false;
            }
            Token::LineComment => {
                if !at_start_of_line && !output.ends_with(' ') && !output.ends_with('\n') {
                    output.push(' ');
                }
                output.push_str(lexer.slice());
                // Comment itself doesn't trigger space because it's usually end of line,
                // but Newline will handle it.
            }
            Token::Newline => unreachable!(),
        }
    }

    Ok(output)
}
