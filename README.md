# c4_rust_AlRafaah: C4 Compiler Rewritten in Rust

**Team:** c4_rust_AlRafaah  
**Project:** Rewrite the original C4 compiler in Rust, matching its functionality and self-hosting capability for a subset of C.

---

## Overview

This repository contains a Rust implementation of the C4 compiler, covering the same C subset as the original C version and preserving self-hosting. We leverage Rust’s safety and modern features to deliver:

- **Lexer**: Tokenizes C4 source (decimal/octal numbers, keywords, operators, literals, comments).  
- **Parser**: Recursive-descent AST builder for globals, enums, functions, control flow, and expressions.  
- **AST (Abstract Syntax Tree)**: Rich `Program`, `Item`, `FuncDef`, `Stmt`, `Expr`, and `Type` structures.  
- **Bytecode Compiler**: Translates AST to a custom stack-based bytecode (`OpCode`, `Instruction`, `Chunk`).  
- **Virtual Machine (VM)**: Executes the bytecode with call frames, stack management, and basic syscalls.

## Key Goals

1. **Functional Equivalence**: Compile and run all C4-supported C code, including `c4.c` itself.  
2. **Rust Idioms**: Use ownership, pattern matching, and `Result`/`Option` for error safety.  
3. **Self-Hosting**: Demonstrate the compiler can compile its own source.  
4. **Unit Testing**: Achieve high coverage (≥70%) for lexer, parser, compiler, and VM.  
5. **Documentation**: Provide code comments (`///`), generated docs via `cargo doc`, and this README.  
6. **Collaboration**: GitHub repo `c4_rust_AlRafaah`, feature branches, pull requests, and clear commit history.

---

## Repository Structure

```

├── src/
│   ├── ast.rs        // AST node definitions and compile traits
│   ├── bytecode.rs   // Bytecode instructions, Chunk, and helper methods
│   ├── lexer.rs      // Lexer producing Token stream, handling whitespace/comments
│   ├── parser.rs     // Parser building AST from tokens
│   ├── vm.rs         // Stack-based VM with call frames and syscalls
│   └── main.rs       // Entry point: reads source, parses, compiles, runs

├── tests/
│   ├── bytecode\_tests.rs      // Unit tests for Chunk methods and VM behavior
│   ├── lexer\_tests.rs         // Tokenization, literals, operators, error handling
│   ├── parser\_tests.rs        // AST construction, grammar coverage
│   ├── vm\_tests.rs            // VM arithmetic and control-flow tests
│   └── self\_host.rs           // Self-hosting test (build & run c4.c)

├── Cargo.toml
├── Cargo.lock
├── c4.c
├── .gitignore
└── README.md                  // This file: build/run/test instructions

````

---

## Getting Started

### 1. Clone the Repository

```bash
git clone https://github.com/Bintaryam/c4_rust_AlRafaah.git
cd c4_rust_AlRafaah
````

### 2. Build the Compiler

```bash
# Ensure Rust 1.65+ is installed via rustup
cargo build --release
```

The compiler binary will be at:

```
./target/release/c4_rust_AlRafaah
```

### 3. Run on C4 Source

```bash
# Compile and execute the original C4 C code
./target/release/c4_rust_AlRafaah path/to/c4.c
```

It should reproduce the behavior of the original C4 compiler, including correct exit codes.

---

## Testing

We use Rust’s built-in test framework. Ensure **≥70% coverage** on critical components.

```bash
# Run all unit and integration tests
cargo test --all
```

* **Lexer Tests** (`lexer_tests.rs`): numbers, identifiers, keywords, operators, literals, comments, errors.
* **Parser Tests** (`parser_tests.rs`): globals/enums/functions, control flow, expressions, indexing/calls.
* **Bytecode & VM Tests** (`bytecode_tests.rs`): `Chunk` methods produce correct `Instruction` variants; VM arithmetic and control-flow.
* **Self-Hosting Test** (`self_host.rs`): builds `c4.c` with GCC then runs it against the Rust compiler to verify identical behavior.
* **VM Tests** (`vm_tests.rs`): additional VM execution scenarios and edge cases.

---

## Generating Documentation

All public modules, structs, and functions are documented with `///` comments. To view the API docs:

```bash
cargo doc --open
```

---

## Collaboration and Workflow

* **Repository Name**: `c4_rust_AlRafaah`
* **Branching Strategy**: Feature branches and pull requests for lexer, parser, VM, etc.
* **Commits**: Descriptive messages (e.g., `Implement lexer`, `Fix VM stack underflow`).
* **CI (optional)**: Use GitHub Actions to `cargo fmt`, `cargo clippy`, and `cargo test` on each PR.
