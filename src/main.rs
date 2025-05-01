use std::env;
use std::fs;
use c4_rust_AlRafaah::parser::Parser;
use c4_rust_AlRafaah::bytecode::Chunk;
use c4_rust_AlRafaah::vm::VM;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <source.c>", args[0]);
        std::process::exit(1);
    }

    let source = fs::read_to_string(&args[1])?;
    let mut parser = Parser::new(&source)?;
    let ast = parser.parse_program()?;

    let mut chunk = Chunk::default();
    ast.compile(&mut chunk)?;

    let mut vm = VM::new();
    let result = vm.run(&chunk);
    println!("Program exited with code {}", result);

    Ok(())
}
