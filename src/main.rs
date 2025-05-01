use std::env;
use std::fs;
use c4_rust_AlRafaah::parser::Parser;
use c4_rust_AlRafaah::bytecode::Chunk;
use c4_rust_AlRafaah::vm::VM;

/// Entry point for the compiler-interpreter tool
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Collect command-line arguments into a vector
    let args: Vec<String> = env::args().collect();

    // Expect exactly one argument (the source file path)
    if args.len() != 2 {
        eprintln!("Usage: {} <source.c>", args[0]); // Print usage error to stderr
        std::process::exit(1); // Exit with error code 1
    }

    // Read the source file content into a string
    let source = fs::read_to_string(&args[1])?;

    // Create a new parser instance for the source code
    let mut parser = Parser::new(&source)?;

    // Parse the source code into an abstract syntax tree (AST)
    let ast = parser.parse_program()?;

    // Compile the AST into bytecode
    let mut chunk = Chunk::default();
    ast.compile(&mut chunk)?;

    // Create and run the virtual machine with the compiled bytecode
    let mut vm = VM::new();
    let result = vm.run(&chunk);

    // Print the final result (exit code of the program)
    println!("Program exited with code {}", result);

    Ok(()) // Return success
}
