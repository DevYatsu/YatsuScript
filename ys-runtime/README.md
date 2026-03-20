# ys-runtime

> The YatsuScript execution engine: VM, Generational GC, and Global Context.

`ys-runtime` is the backend responsible for taking compiled YatsuScript bytecode and running it. It manages the runtime state, objects, memory, and native function integrations.

## Why This Exists

This crate decouples execution logic from the language frontend. It implements a **register-based virtual machine** that handles asynchronous execution via bit-packed instructions.

## Quick Start

Add `ys-runtime` to your `Cargo.toml`:

```rust
use ys_core::parser::Parser;
use ys_runtime::run_interpreter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = "let x: (1..10).map(|i| i * 2)\nprint(x)";
    let program = Parser::new(source)?.compile()?;
    
    // Execute the compiled program
    run_interpreter(program).await?;
    
    Ok(())
}
```

## Features

- **Generational GC**: A high-performance, concurrent garbage collector with nursery and tenured generations.
- **NaN-boxed Runtime**: Fast, header-less 64-bit value representation for all types.
- **Asynchronous Execution**: Native `spawn` support via Tokio for true parallel task processing.
- **Native Integration**: Easy-to-use hooks for registering custom Rust functions as YatsuScript built-ins.

## Built-in Modules

- **[I/O](src/natives/io.rs)**: `print()`, `str()`.
- **[Network](src/natives/net.rs)**: `fetch()`, `serve()`.
- **[System](src/natives/mod.rs)**: `len()`, `sleep()`, and more.

## Architecture

1. **[Virtual Machine](src/vm/mod.rs)**: The core dispatch loop that executes `Instruction` variants.
2. **[Heap Manager](src/heap.rs)**: Object allocation and GC trace-and-sweep logic.
3. **[Global Context](src/context.rs)**: Shared state, string interning pool, and global variables.
4. **[Value Helpers](src/value_fmt.rs)**: Logic for pretty-printing results and values (including heap-resident objects).

## License

MIT © Yanis
