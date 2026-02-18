mod compiler;
mod error;
mod lexer;
mod parser;
mod vm;

use crate::error::JitError;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), JitError> {
    let content = std::fs::read_to_string("./main.pi")
        .map_err(|e| JitError::Runtime(format!("Failed to read file: {}", e), 0, 0))?;

    let parser = parser::Parser::new(&content);
    let program = match parser.compile() {
        Ok(prog) => prog,
        Err(e) => {
            print_error(&content, &e);
            std::process::exit(1);
        }
    };

    let start = Instant::now();

    // Run the program on the Tokio runtime
    vm::run(program).await?;

    let total = start.elapsed();

    println!("\n--- Execution Results (Tokio Green Threads) ---");
    println!("Total execution time: {:?}", total);
    println!("-----------------------------------------------");

    Ok(())
}

fn print_error(source: &str, error: &JitError) {
    eprintln!("\x1b[31;1mError:\x1b[0m {}", error);
    let (line_num, col_num) = error.location();
    if line_num > 0 {
        let lines: Vec<&str> = source.lines().collect();
        if line_num <= lines.len() {
            let line_content = lines[line_num - 1];
            eprintln!("\n\x1b[34m{:>4} | \x1b[0m{}", line_num, line_content);
            let padding = " ".repeat(col_num.saturating_sub(1));
            eprintln!("\x1b[34m     | \x1b[0m{}\x1b[31;1m^\x1b[0m", padding);
        }
    }
}
