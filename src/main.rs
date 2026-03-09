//! # small_interpreter
//!
//! A fast bytecode interpreter for the **Pi** scripting language.
//!
//! ## Features
//!
//! - **NaN-boxing** — all runtime values fit in a single `u64`, keeping the
//!   hot path allocation-free for numbers, booleans, and short strings (≤ 6 bytes).
//! - **GC** — a minor/major two-generation garbage collector with
//!   a remembered set and write-barrier, enabling high allocation throughput.
//! - **Async concurrency** — `spawn { … }` blocks run as Tokio tasks on the
//!   current thread-pool; native async I/O (`fetch`, `serve`, `sleep`) comes
//!   built-in.
//! - **Register-based bytecode** — the parser compiles directly to a compact
//!   instruction set; there is no AST walk at runtime.
//!
//! ## Usage
//!
//! ```text
//! # Run a .pi file
//! small_interpreter script.pi
//!
//! # Start the interactive REPL (no argument)
//! small_interpreter
//! ```

mod backends;
mod compiler;
mod error;
mod formatter;
mod lexer;
mod parser;

#[cfg(test)]
mod tests;

use crate::backends::Backend;
use crate::error::JitError;
use mimalloc::MiMalloc;
use std::time::Instant;

/// Use mimalloc as the global allocator for superior allocation throughput,
/// especially under the GC-heavy workloads typical of dynamic languages.
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

/// Entry point.  Two modes of operation:
///
/// 1. **Script mode** — a file path is given as the first positional argument.
///    The file is read, compiled, and executed.
/// 2. **REPL mode** — no file path is given.  A read-eval-print loop starts,
///    similar to running `python` or `node` with no arguments.
#[tokio::main]
async fn main() -> Result<(), JitError> {
    let mut args = pico_args::Arguments::from_env();

    if args.contains(["-h", "--help"]) {
        print_usage();
        return Ok(());
    }

    let subcommand = args.subcommand().map_err(|e| {
        JitError::Runtime(format!("Failed to parse subcommand: {}", e), 0, 0)
    })?;

    match subcommand.as_deref() {
        Some("fmt") => {
            let files: Vec<String> = args.finish().into_iter().map(|s| s.to_string_lossy().into_owned()).collect();
            run_fmt(files).await
        }
        _ => {
            // If a file path was provided, run it; otherwise start the REPL.
            match args.free_from_str::<String>() {
                Ok(file_path) => run_file(&file_path).await,
                Err(_) => run_repl().await,
            }
        }
    }
}

use std::fs;
use std::path::Path;

/// Format .pi files.
async fn run_fmt(files: Vec<String>) -> Result<(), JitError> {
    if files.is_empty() {
        // Format current project recursively
        format_recursive(Path::new("."))?;
    } else {
        for file in files {
            let path = Path::new(&file);
            if path.is_dir() {
                format_recursive(path)?;
            } else {
                formatter::format_file(path)?;
            }
        }
    }
    Ok(())
}

fn format_recursive(path: &Path) -> Result<(), JitError> {
    if path.is_dir() {
        for entry in fs::read_dir(path).map_err(|e| JitError::Runtime(e.to_string(), 0, 0))? {
            let entry = entry.map_err(|e| JitError::Runtime(e.to_string(), 0, 0))?;
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().map(|n| n == "target" || n == ".git").unwrap_or(false) {
                    continue;
                }
                format_recursive(&path)?;
            } else if path.extension().map(|e| e == "pi").unwrap_or(false) {
                formatter::format_file(&path)?;
            }
        }
    } else if path.extension().map(|e| e == "pi").unwrap_or(false) {
        formatter::format_file(path)?;
    }
    Ok(())
}

/// Run a `.pi` source file from disk.
async fn run_file(file_path: &str) -> Result<(), JitError> {
    let content = std::fs::read_to_string(file_path).map_err(|e| {
        JitError::Runtime(format!("Failed to read file '{}': {}", file_path, e), 0, 0)
    })?;

    let program = match parser::Parser::new(&content).compile() {
        Ok(prog) => prog,
        Err(e) => {
            print_error(&content, &e);
            std::process::exit(1);
        }
    };

    if cfg!(debug_assertions) {
        println!("Starting execution…");
    }

    let start = Instant::now();
    let backend: Box<dyn Backend> = Box::new(backends::interpreter::Interpreter);

    if let Err(e) = backend.run(program).await {
        print_error(&content, &e);
        std::process::exit(1);
    }

    if cfg!(debug_assertions) {
        println!("\nExecution completed in {:?}", start.elapsed());
    }

    Ok(())
}

/// Interactive REPL — reads one or more lines of Pi source, executes them,
/// then loops.  Multi-line input is collected until either a blank line is
/// entered (to flush the buffer) or the buffer parses cleanly as a complete
/// program (no trailing open braces/brackets).
///
/// Type `exit` or `quit` (or send EOF with Ctrl-D) to leave.
async fn run_repl() -> Result<(), JitError> {
    // Print the welcome banner.
    println!(
        "\x1b[1;36msmall_interpreter\x1b[0m {} — Pi language REPL",
        env!("CARGO_PKG_VERSION")
    );
    println!("Type \x1b[33mexit\x1b[0m or \x1b[33mquit\x1b[0m to leave, or press Ctrl-D.");
    println!();

    let mut rl = rustyline::DefaultEditor::new()
        .map_err(|e| JitError::Runtime(format!("Failed to initialize REPL: {}", e), 0, 0))?;

    let history_file = ".pi_history";
    let _ = rl.load_history(history_file);

    let mut buffer = String::new(); // accumulated multi-line source
    let mut continuation = false; // are we inside an open block?

    loop {
        // Choose the prompt based on whether we are mid-block.
        let prompt = if continuation {
            "\x1b[1;32m... \x1b[0m"
        } else {
            "\x1b[1;32m>>> \x1b[0m"
        };

        let readline = rl.readline(prompt);
        match readline {
            Ok(line) => {
                let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');

                // Handle exit commands.
                if !continuation && matches!(trimmed, "exit" | "quit") {
                    println!("Bye!");
                    break;
                }

                buffer.push_str(trimmed);
                buffer.push('\n');

                // Count unmatched `{` to decide if we need more input.
                let opens: i32 = buffer.chars().filter(|&c| c == '{').count() as i32;
                let closes: i32 = buffer.chars().filter(|&c| c == '}').count() as i32;
                continuation = opens > closes;

                // Execute when the block is balanced
                if !continuation {
                    let src = buffer.trim().to_string();
                    if !src.is_empty() {
                        let _ = rl.add_history_entry(&src); // save full block to history
                        eval_and_print(src).await;
                    }
                    buffer.clear();
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                // Ctrl-C cancels the current multi-line block without exiting REPL
                println!("^C");
                buffer.clear();
                continuation = false;
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                // Ctrl-D
                println!("Bye!");
                break;
            }
            Err(err) => {
                eprintln!("REPL error: {:?}", err);
                break;
            }
        }
    }

    let _ = rl.save_history(history_file);
    Ok(())
}

/// Compile and run a snippet of Pi source, printing any
/// errors inline. The top-level execution is awaited so immediate outputs
/// appear before the next REPL prompt, but spawned tasks continue in the background.
async fn eval_and_print(source: String) {
    let program = match parser::Parser::new(&source).compile() {
        Ok(prog) => prog,
        Err(e) => {
            print_error(&source, &e);
            return;
        }
    };

    let backend: Box<dyn crate::backends::Backend + Send> =
        Box::new(crate::backends::interpreter::Interpreter);
    if let Err(e) = backend.run(program).await {
        print_error(&source, &e);
    }
}

/// Print a short usage message to stdout.
fn print_usage() {
    println!(
        "\
\x1b[1msmall_interpreter\x1b[0m — Pi scripting language

\x1b[1;33mUSAGE\x1b[0m
    small_interpreter [FILE]
    small_interpreter fmt [FILES...]

\x1b[1;33mARGS\x1b[0m
    FILE    Path to a .pi source file.  If omitted, starts the interactive REPL.
    FILES   Paths to .pi files or directories to format.

\x1b[1;33mFLAGS\x1b[0m
    -h, --help    Print this message and exit

\x1b[1;33mEXAMPLES\x1b[0m
    small_interpreter script.pi
    small_interpreter                 # interactive REPL
    small_interpreter fmt             # format current project
    small_interpreter fmt script.pi   # format a specific file
"
    );
}

/// Pretty-print a [`JitError`] to stderr, including the offending source line
/// with a caret (`^`) pointing at the column where the error occurred.
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
