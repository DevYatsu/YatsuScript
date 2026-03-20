use std::path::Path;
use std::fs;
use std::time::Instant;

use ys_core::parser::Parser;
use ys_runtime::{run_interpreter, Interpreter};

/// Run a script file.
pub async fn run_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let source = fs::read_to_string(path)?;
    run_source(&source).await
}

/// Run source code directly.
pub async fn run_source(source: &str) -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();

    let parser = Parser::new(source)?;
    let program = parser.compile()?;
    
    let elapsed_compile = start.elapsed();
    
    let start_run = Instant::now();
    run_interpreter(program).await?;
    let elapsed_run = start_run.elapsed();

    println!("\n{} {:?} (compile: {:?}, run: {:?})", 
        "Done in", start.elapsed(), elapsed_compile, elapsed_run);
    
    Ok(())
}

/// Start an interactive REPL session.
pub async fn run_repl() -> Result<(), Box<dyn std::error::Error>> {
    let mut rl = rustyline::DefaultEditor::new()?;
    let _runtime = Interpreter;

    println!("YatsuScript REPL (press Ctrl-C to exit)");
    
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                
                if line.trim().is_empty() { continue; }

                let parser = match Parser::new(&line) {
                    Ok(p) => p,
                    Err(e) => {
                        crate::error_display::display_error(&e, &line);
                        continue;
                    }
                };

                let program = match parser.compile() {
                    Ok(p) => p,
                    Err(e) => {
                        crate::error_display::display_error(&e, &line);
                        continue;
                    }
                };

                if let Err(e) = run_interpreter(program).await {
                    crate::error_display::display_error(&e, &line);
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted) => break,
            Err(rustyline::error::ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    
    Ok(())
}
