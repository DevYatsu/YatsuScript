use colored::Colorize;
use ys_core::error::JitError;

/// Pretty-print a YatsuScript error with syntax highlighting logic.
pub fn display_error(err: &JitError, source: &str) {
    let (msg, line, col) = match err {
        JitError::Lexing { err, loc } => (err.to_string(), loc.line as usize, loc.col as usize),
        JitError::Parsing { msg, loc } => (msg.clone(), loc.line as usize, loc.col as usize),
        JitError::Runtime { msg, loc } => (msg.clone(), loc.line as usize, loc.col as usize),
        JitError::UnknownVariable { msg, loc } => (msg.clone(), loc.line as usize, loc.col as usize),
        JitError::RedefinitionOfImmutableVariable { msg, loc, orig_line } => {
            (format!("{} (already defined on line {})", msg, orig_line), loc.line as usize, loc.col as usize)
        }
    };

    println!("\n{} {} line {}, column {}", "Error: ".red().bold(), msg.white(), line, col);

    let lines: Vec<&str> = source.lines().collect();
    if line > 0 && line <= lines.len() {
        let error_line = lines[line - 1];
        println!("  {:3} | {}", line, error_line);
        let padding = " ".repeat(line.to_string().len() + 5 + col - 1);
        println!("{}^ here", padding.red().bold());
    }
}
