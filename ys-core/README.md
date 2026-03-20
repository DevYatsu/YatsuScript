# ys-core

> The linguistic frontend of YatsuScript: Lexer, Parser, and Bytecode Compiler.

`ys-core` is the heart of the YatsuScript language. It transforms raw ASCII source code into optimized, register-based bytecode ready for execution by the runtime.

## Features

- **Blazing Fast Lexing**: Built on top of the `logos` crate for DFA-based scanning.
- **Hand-written Parser**: A recursive descent parser with clean error reporting.
- **Register-based Bytecode**: Compiles high-level constructs into a compact instruction set.
- **NaN-Boxing**: Efficient value representation using 64-bit NaN-boxed words.

## Usage

Add `ys-core` to your `Cargo.toml`:

```toml
[dependencies]
ys-core = { path = "../ys-core" }
```

### Compiling Source to Bytecode

```rust
use ys_core::parser::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = "let x: 10\nprint(x + 5)";
    
    // 1. Initialize the parser
    let parser = Parser::new(source)?;
    
    // 2. Compile to a Program
    let program = parser.compile()?;
    
    println!("Compiled {} instructions.", program.instructions.len());
    Ok(())
}
```

## Internal Architecture

1. **[Lexer](src/lexer.rs)**: Tokenizes the input string into a stream of `Token` variants.
2. **[Parser](src/parser.rs)**: Consumes tokens and performs syntactic analysis.
3. **[Compiler](src/compiler.rs)**: Generates `Instruction` sequences and manages the string pool.
4. **[Error Handling](src/error.rs)**: Provides structured `JitError` types with precise source locations.

## License

MIT © Yanis
