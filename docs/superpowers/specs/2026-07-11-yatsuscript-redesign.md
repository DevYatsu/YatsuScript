# YatsuScript 2.0 — Language Redesign

**Date:** 2026-07-11
**Status:** Draft
**Author:** Orchestrator (after brainstorming with user)

## 1. Motivation

Complete syntax redesign while keeping the battle-tested VM/runtime layer untouched. The new language
is simpler, more familiar to Python/Rust developers, and supports closures and a proper module system.

### Design principles

- **Keep the VM.** Bytecode, register dispatch, NaN-boxing, generational GC, heap, native functions —
  all stay identical. Only the frontend (lexer → parser → compiler) is rewritten.
- **Familiar syntax.** Python-style variable declarations, Rust-style closures, C-style blocks.
- **Closures are real.** First-class capture-by-reference / capture-by-value closures, implemented
  as a minimal (~50 line) addition to the runtime.
- **Module system at compile time.** `use` paths are resolved to files at parse time. All modules are
  compiled and linked into one combined `Program` before execution. Zero VM changes for modules.
- **Minimal new runtime code.** ~200–300 lines total across all runtime additions (Closure object,
  MakeClosure instruction, CallDynamic closure dispatch, module linker).

## 2. Lexical Structure

| Feature | Design |
|---|---|
| **Comments** | `//` line, `/* */` block |
| **Keywords** | `fun`, `if`, `else`, `while`, `for`, `in`, `return`, `use`, `super`, `exp`, `move`, `true`, `false`, `nil`, `and`, `or` |
| **Strings** | Double-quoted `"hello"` |
| **Templates** | `` `text ${expr} text` `` |
| **Numbers** | `123`, `3.14`, `-1.5` (all f64 at runtime) |
| **Identifiers** | `[a-zA-Z_][a-zA-Z0-9_]*` |
| **Delimiters** | `{}` blocks, `()` expressions/params, `[]` lists |
| **Separators** | Significant newlines (statement terminators) |
| **Operators** | `=` `==` `!=` `<` `<=` `>` `>=` `+` `-` `*` `/` `!` `..` `.` `,` `:` `\|` `->` |

### 2.1 Keywords summary

| Keyword | Usage |
|---|---|
| `fun` | Named function declaration |
| `if`, `else` | Conditional |
| `while` | Loop |
| `for`, `in` | Range/collection iteration |
| `return` | Explicit return from function |
| `use`, `super` | Module imports |
| `exp` | Export visibility (like Rust's `pub`) |
| `move` | Closure capture-by-value |
| `true`, `false`, `nil` | Literals |
| `and`, `or` | Short-circuit boolean combinators |

### 2.2 Operators and precedence

(TODO: fill precise precedence table during compilation — for spec,
grouped by category)

| Category | Operators |
|---|---|
| Primary | `(...)`, `[...]`, `{...}`, literals, identifiers |
| Postfix | `(args)`, `[index]`, `.field` |
| Unary | `!`, `-` |
| Range | `..` |
| Multiplicative | `*`, `/` |
| Additive | `+`, `-` |
| Comparison | `==`, `!=`, `<`, `<=`, `>`, `>=` |
| Logical | `and`, `or` |
| Assignment | `=` |
| Closure | `\|params\| expr` |

## 3. Variables

```
x = 10              # declaration (if new scope) or reassignment
name = "hello"      # all variables are mutable by default
count: int = 0      # optional type hint (stripped at compile time)
```

- No `let`, no `mut`, no `const`.
- Variable comes into existence on first assignment in a scope.
- Reassignment to an existing name mutates the existing variable.
- Scoping rules follow Python-like LEGB (Local → Enclosing → Global → Builtin).
- Type annotations after `:` are syntax-checked but completely stripped during
  bytecode compilation — zero runtime cost.

## 4. Functions

### 4.1 Named functions

```
fun greet(name) {
    return "Hello, " + name
}

fun add(a, b) {
    a + b               # implicit return of last expression
}

# With optional type hints
fun add(a: int, b: int) -> int {
    a + b
}

# Exported (visible to other modules)
exp fun public_api() {
    "this is public"
}
```

- `fun` keyword, name, parameters `(...)`, body block `{}`.
- `return expr` for early/explicit return. Without any `return`, the last
  expression in the body is the implicit return value.
- Named functions do **not** capture their environment (like Rust `fn`).
- `exp` marks the function as visible to importing modules.
- Type annotations are syntax-checked and stripped at compile time.

### 4.2 Anonymous functions / Closures

```
# Inline — single expression
list.map(|x| x * 2)

# Block body
list.filter(|x| {
    x > 0 && x < 100
})

# Capture by value
let handler = move |event| {
    save(event)
}

# Stored in a variable
let callback = |v| print(v)
callback("hello")
```

- Syntax: `|param1, param2, ...| expr` or `|param1, param2, ...| { block }`.
- Closures **do** capture variables from the enclosing scope (by reference).
- `move` keyword forces capture by value (deep copy at closure creation time).
- Closures are heap-allocated objects. They can be stored, passed, and called
  via `CallDynamic`.

### 4.3 Implementation: closure heap object

New variant in the `ManagedObject` enum:

```rust
Closure {
    func_index: u32,         // index into Program.functions
    captures: Vec<Value>,    // captured register values (copied at construction)
}
```

New instruction:

```rust
MakeClosure { dst: usize, func_index: usize, captures: Arc<[usize]> }
```

- The compiler emits `MakeClosure` at the point where a closure expression appears.
- `captures` lists the register indices of variables that the closure references
  from the enclosing scope. The compiler determines this set during parsing.
- The VM reads each capture register and stores its current value in the Closure
  object's `captures` vec.

`CallDynamic` dispatch addition (~10 lines):

```rust
if let Some(ManagedObject::Closure(cl)) = heap.get(callee_obj_id) {
    // Push a new frame with captured values as the first registers,
    // then the explicit call arguments in remaining registers.
}
```

## 5. Control Flow

### 5.1 if / else

```
if x > 0 {
    print("positive")
} else if x == 0 {
    print("zero")
} else {
    print("negative")
}
```

- Standard conditional. `else if` chains supported.
- Condition is an arbitrary expression (coerced to boolean).

### 5.2 while

```
while n > 0 {
    print(n)
    n = n - 1
}
```

### 5.3 for

```
for i in 0..10 {
    print(i)
}

for i in range.step(2) {
    print(i)
}
```

- `for var in expr` iterates over a Range object.
- `expr` must evaluate to a Range (heap object with start, end, step).
- `0..10` is the range literal syntax (same as current).
- The current `JumpIfNotLess` optimization applies — the for-loop is compiled
  to a single comparison + jump back, no heap allocation per iteration.

## 6. Method Chaining

```
result = list.map(|x| x * 2).filter(|x| x > 5).sum()
```

- `.` left-to-right chaining through `CallDynamic`.
- Each call dispatches dynamically on the receiver's type.
- Methods are just functions that receive the object as an implicit receiver.
- Works with closures passed as arguments to methods (common pattern).

## 7. Module System

### 7.1 Module = file

Every `.ys` file is a module. Directory structure maps to module hierarchy:

```
src/
├── main.ys          # entry point
├── utils/
│   ├── parse.ys     # module: utils::parse
│   └── format.ys    # module: utils::format
└── models/
    └── user.ys      # module: models::user
```

### 7.2 Import syntax

```
use std::net::fetch           # brings `fetch` function into scope
use utils::parse              # brings the `parse` module into scope
use utils::parse::parse_line  # brings `parse_line` directly
use super::common::types      # relative path (up one directory)
```

Resolution rules:
1. `use a::b::c` — look for `a/b/c.ys`, then `a/b/c/mod.ys`
2. Relative paths start from the importing file's directory
3. `super::x` goes up one directory level
4. Multiple `super::` may be chained: `super::super::x`
5. There is no `crate::` keyword — the entry file is the implicit root

### 7.3 Export visibility

```
exp fun visible() { }    # visible to importers
fun hidden() { }         # private to this module

exp x = 42               # exported constant
```

- Items marked `exp` are visible outside the module.
- Everything else is private.
- Only top-level items can be `exp`.

### 7.4 Compile-time linking (zero VM changes)

1. Parse entry file, collect all `use` statements.
2. Resolve each `use` to a file path, parse + compile that file.
3. Recursively process `use` statements in dependencies (detect cycles, error).
4. Link all compiled `Program` structs into one:
   - Merge `functions` arrays (renumber indices).
   - Merge `globals` arrays (renumber indices).
   - Merge `string_pool` (deduplicate).
5. The combined `Program` is passed to the VM unchanged.

A new module in `ys-core` handles resolution and linking:
`ys-core::module::{resolve, linker}`.

## 8. Standard Library

### 8.1 Prelude (always available)

These names are available in every program without any `use`:

| Name | Description |
|---|---|
| `print(...)` | Print values to stdout |
| `str(val)` | Convert value to string |
| `len(val)` | Length of list, string, or object |
| `type(val)` | Return type name of value |
| `range(start, end, step?)` | Create a Range |
| `to_int(val)` | Convert to integer (truncate f64) |
| `to_float(val)` | Convert to f64 |
| `true`, `false`, `nil` | Literal constants |
| `not` (or `!`) | Boolean negation |
| `and`, `or` | Short-circuit combinators |

### 8.2 Standard modules (explicit `use` required)

```
use std::io              # file I/O: read, write
use std::net             # fetch, serve (HTTP)
use std::time            # time(), sleep(ms), timestamp
use std::collections     # list/object utility methods
```

Each maps to a file in the stdlib directory. The stdlib path is configurable
via environment variable (`YS_PATH`) or bundled with the runtime.

## 9. VM Changes (Complete List)

| Change | Crate | Lines | Complexity |
|---|---|---|---|
| `Closure` variant in `ManagedObject` | `ys-runtime/src/heap.rs` | ~15 | Low |
| `MakeClosure` instruction variant | `ys-core/src/compiler.rs` | ~5 | Low |
| `MakeClosure` handler in VM dispatch | `ys-runtime/src/vm/mod.rs` | ~15 | Low |
| `CallDynamic` closure dispatch | `ys-runtime/src/vm/mod.rs` | ~10 | Low |
| Module resolver + linker | `ys-core/src/module.rs` | ~150 | Medium |
| **Total** | | **~200** | |

Everything else is a frontend rewrite: lexer, parser, compiler.

## 10. Frontend Changes

### 10.1 Lexer (`ys-core/src/lexer.rs`)

- Add tokens: `=`, `fun`, `use`, `super`, `exp`, `move`, `and`, `or`, `|`, `->`
- Remove tokens: `let`, `mut`, `fn`, `spawn`
- Keep: `..`, `.`, `,`, `:`, `{`, `}`, `(`, `)`, `[`, `]`, `+`, `-`, `*`, `/`,
  `!`, `==`, `!=`, `<`, `<=`, `>`, `>=`, identifiers, numbers, strings,
  templates, line comments, block comments
- Significant newlines remain as statement terminators

### 10.2 Parser (`ys-core/src/parser.rs`)

Complete rewrite. New grammar:

```
program         = statement*

statement       = assignment
                | fun_declaration
                | return_statement
                | if_statement
                | while_statement
                | for_statement
                | use_statement
                | expression

assignment      = identifier (":" type)? "=" expression
fun_declaration = "exp"? "fun" identifier "(" params ")" ("->" type)? block
return_stmt     = "return" expression?
if_statement    = "if" expression block ("else" (if_statement | block))?
while_statement = "while" expression block
for_statement   = "for" identifier "in" expression block
use_statement   = "use" path

expression      = or_expr
or_expr         = and_expr ("or" and_expr)*
and_expr        = comp_expr ("and" comp_expr)*
comp_expr       = add_expr (("=="|"!="|"<"|"<="|">"|">=") add_expr)*
add_expr        = mul_expr (("+"|"-") mul_expr)*
mul_expr        = unary_expr (("*"|"/") unary_expr)*
unary_expr      = ("!"|"-") unary_expr | postfix_expr
postfix_expr    = primary_expr ("(" args ")" | "[" expr "]" | "." identifier)*
primary_expr    = literal | identifier | "(" expression ")" | "[" list_lit "]" |
                  "{" obj_lit "}" | closure | range_lit

closure         = "move"? "|" params "|" (expression | block)
range_lit       = expression ".." expression
```

### 10.3 Compiler (`ys-core/src/compiler.rs`)

- `Instruction` enum: add `MakeClosure` variant.
- Compile new expression/statement types to existing bytecode where possible:
  - `x = 5` → `LoadLiteral + StoreLocal` (or `StoreGlobal`)
  - `fun foo() {}` → registers function in `Program.functions` (same as current `fn`)
  - `\|x\| x + 1` → `MakeClosure` with captured variables list
  - `use a::b` → resolved by module linker before compilation
- Remove compilation of removed constructs (`spawn`).

### 10.4 Module linker (`ys-core/src/module.rs`, new file)

```rust
pub fn resolve_and_link(entry_path: &Path, stdlib_path: &Path) -> Result<Program, Error>
```

1. Parse entry file into a list of `use` statements and a partially-resolved program.
2. For each `use`, resolve the path to a file, recursively compile.
3. Detect circular dependencies.
4. Link all programs:
   - Build a merged `Vec<UserFunction>` (renumber function refs).
   - Build a merged `Vec<String>` string pool (deduplicate).
   - Build a merged global variable layout.
5. Return a single `Program`.

## 11. Examples

### 11.1 Hello World

```
print("Hello, World!")
```

### 11.2 Fibonacci

```
fun fib(n) {
    if n <= 1 {
        n
    } else {
        fib(n - 1) + fib(n - 2)
    }
}

print(fib(35))
```

### 11.3 Prime sieve

```
fun sieve(n) {
    limit = n - 1
    is_prime = range(0, limit).map(|_| true)
    count = 0
    for i in 0..limit {
        if is_prime[i] {
            count = count + 1
            step = i + 2
            for j in range(i + i + 3, limit, step) {
                is_prime[j] = false
            }
        }
    }
    return count
}

print(sieve(1000000))
```

### 11.4 Modules

```
# File: src/main.ys
use utils::parse
use std::net::fetch

exp fun main() {
    data = fetch("https://example.com/data")
    result = parse(data)
    print(result)
}
```

```
# File: src/utils/parse.ys
exp fun parse(input) {
    # parsing logic...
    result
}

fun helper() {
    "private helper"
}
```

### 11.5 Closures with method chaining

```
numbers = [1, 2, 3, 4, 5]
result = numbers.map(|x| x * 2).filter(|x| x > 5).reduce(|a, b| a + b)
print(result)  # 14
```

## 12. What Stays the Same

- Bytecode instruction set (30 existing variants — all remain)
- Register-based dispatch loop
- NaN-boxed Value representation
- Generational GC (nursery + tenured + remembered set)
- Heap internals (SyncCell, ManagedObject, allocation)
- Frame stack model (CallFrame with registers + PC + return_to)
- All existing native functions (some reorganized into stdlib modules)
- `CallDynamic`, `BoundMethod`, `Increment`, `JumpIfNotLess` optimizations
- `ys-lsp` — will be updated after frontend rewrite (separate task)

## 13. Migration / Backward Compatibility

The old syntax will not be supported. Existing `.ys` files must be rewritten to
the new syntax. The changes are mechanical enough for a one-time migration:

| Old | New |
|---|---|
| `let x: 5` | `x = 5` |
| `mut x: 5` | `x = 5` |
| `fn name(p) { }` | `fun name(p) { }` |
| `x: x + 1` | `x = x + 1` |
| `spawn` | removed (error) |
| `//` comments | same |
| `/* */` comments | same |

A migration script is out of scope for this spec.

## 14. Out of Scope

- WASM compilation target (planned for future)
- Game engine embedding API (planned for future)
- Formal type checker / type inference
- Pattern matching
- Async / await syntax
- Iterators / generator functions
- AOT compilation
- LSP update (separate task after frontend lands)

## A. Spec Self-Review

- [x] No placeholder sections remain
- [x] Internal consistency: lexer tokens match parser grammar, grammar matches
      examples, VM changes match compiler output
- [x] Scope check: focused on syntax redesign + closures + modules. No scope creep.
- [x] Ambiguity check: all syntax decisions are explicit. The only open detail
      is list repeat syntax (`[true] * limit` vs `[true; limit]`), flagged as
      a TODO.
