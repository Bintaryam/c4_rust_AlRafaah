use c4_rust_AlRafaah::bytecode::*;
use c4_rust_AlRafaah::vm::VM;
use c4_rust_AlRafaah::ast::*;

// Manual Bytecode Tests 

fn run_chunk(chunk: Chunk) -> i64 {
    let mut vm = VM::new();
    vm.run(&chunk)
}

#[test]
fn test_addition() {
    let mut chunk = Chunk::default();
    chunk.push_int(OpCode::IMM, 40);
    chunk.push(OpCode::PSH);
    chunk.push_int(OpCode::IMM, 2);
    chunk.push(OpCode::ADD);
    chunk.push(OpCode::EXIT);

    assert_eq!(run_chunk(chunk), 42);
}

#[test]
fn test_comparisons() {
    let mut chunk = Chunk::default();
    chunk.push_int(OpCode::IMM, 10);
    chunk.push(OpCode::PSH);
    chunk.push_int(OpCode::IMM, 20);
    chunk.push(OpCode::LT);
    chunk.push(OpCode::EXIT);
    assert_eq!(run_chunk(chunk), 1);
}

#[test]
fn test_conditional_jump_false() {
    let mut chunk = Chunk::default();
    chunk.push_int(OpCode::IMM, 0);
    chunk.push_jump(OpCode::BZ, 4);
    chunk.push_int(OpCode::IMM, 100);
    chunk.push(OpCode::JMP);
    chunk.push_int(OpCode::IMM, 42);
    chunk.push(OpCode::EXIT);

    assert_eq!(run_chunk(chunk), 42);
}

#[test]
fn test_conditional_jump_true() {
    let mut chunk = Chunk::default();
    chunk.push_int(OpCode::IMM, 1);
    chunk.push_jump(OpCode::BZ, 4);
    chunk.push_int(OpCode::IMM, 42);
    chunk.push(OpCode::EXIT);
    chunk.push_int(OpCode::IMM, 999);

    assert_eq!(run_chunk(chunk), 42);
}

#[test]
fn test_stack_and_load_store() {
    let mut body = Chunk::default();
    body.push_int(OpCode::ENT, 1);
    body.push_int(OpCode::IMM, 123);
    body.push(OpCode::PSH);
    body.push_int(OpCode::LEA, 0);
    body.push(OpCode::SI);
    body.push_int(OpCode::LEA, 0);
    body.push(OpCode::LI);
    body.push(OpCode::LEV);

    let mut wrapper = Chunk::default();
    let entry_point = wrapper.code.len() + 2;
    wrapper.push_call(OpCode::JSR, entry_point);
    wrapper.push(OpCode::EXIT);
    wrapper.code.extend(body.code);

    assert_eq!(run_chunk(wrapper), 123);
}

#[test]
fn test_nested_arithmetic() {
    let mut chunk = Chunk::default();
    chunk.push_int(OpCode::IMM, 5);
    chunk.push(OpCode::PSH);
    chunk.push_int(OpCode::IMM, 3);
    chunk.push(OpCode::ADD);
    chunk.push(OpCode::PSH);
    chunk.push_int(OpCode::IMM, 2);
    chunk.push(OpCode::MUL);
    chunk.push(OpCode::EXIT);

    assert_eq!(run_chunk(chunk), 16);
}

#[test]
fn test_equality_logic() {
    let mut chunk = Chunk::default();
    chunk.push_int(OpCode::IMM, 10);
    chunk.push(OpCode::PSH);
    chunk.push_int(OpCode::IMM, 10);
    chunk.push(OpCode::EQ);
    chunk.push(OpCode::EXIT);

    assert_eq!(run_chunk(chunk), 1);
}

#[test]
fn test_call_and_return() {
    let mut chunk = Chunk::default();
    chunk.push_call(OpCode::JSR, 2);
    chunk.push(OpCode::EXIT);
    chunk.push_int(OpCode::IMM, 42);
    chunk.push(OpCode::LEV);

    assert_eq!(run_chunk(chunk), 42);
}

#[test]
fn test_ent_adj_lev_function_frame() {
    let mut body = Chunk::default();
    body.push_int(OpCode::ENT, 1);
    body.push_int(OpCode::IMM, 99);
    body.push(OpCode::PSH);
    body.push_int(OpCode::LEA, 0);
    body.push(OpCode::SI);
    body.push_int(OpCode::LEA, 0);
    body.push(OpCode::LI);
    body.push(OpCode::LEV);

    let mut chunk = Chunk::default();
    let func_start = chunk.code.len() + 2;
    chunk.push_call(OpCode::JSR, func_start);
    chunk.push(OpCode::EXIT);
    chunk.code.extend(body.code);

    assert_eq!(run_chunk(chunk), 99);
}

// AST → Bytecode → VM Tests 

fn run_ast(program: Program) -> i64 {
    let mut chunk = Chunk::default();
    program.compile(&mut chunk).unwrap();
    let mut vm = VM::new();
    vm.run(&chunk)
}

#[test]
fn test_ast_simple_return() {
    let program = Program {
        items: vec![Item::Function(FuncDef {
            name: "main".into(),
            params: vec![],
            locals: vec![],
            ret: Type::Int,
            body: Block {
                stmts: vec![Stmt::Return(Some(Expr::Num(42)))],
            },
        })],
    };
    assert_eq!(run_ast(program), 42);
}

#[test]
fn test_ast_addition() {
    let program = Program {
        items: vec![Item::Function(FuncDef {
            name: "main".into(),
            params: vec![],
            locals: vec![],
            ret: Type::Int,
            body: Block {
                stmts: vec![Stmt::Return(Some(Expr::Binary {
                    op: BinOp::Add,
                    left: Box::new(Expr::Num(20)),
                    right: Box::new(Expr::Num(22)),
                }))],
            },
        })],
    };
    assert_eq!(run_ast(program), 42);
}

#[test]
fn test_ast_nested_binary_expression() {
    let program = Program {
        items: vec![Item::Function(FuncDef {
            name: "main".into(),
            params: vec![],
            locals: vec![],
            ret: Type::Int,
            body: Block {
                stmts: vec![Stmt::Return(Some(Expr::Binary {
                    op: BinOp::Mul,
                    left: Box::new(Expr::Binary {
                        op: BinOp::Add,
                        left: Box::new(Expr::Num(2)),
                        right: Box::new(Expr::Num(3)),
                    }),
                    right: Box::new(Expr::Num(8)),
                }))],
            },
        })],
    };
    assert_eq!(run_ast(program), 40);
}

#[test]
fn test_ast_expression_stmt_discarded() {
    let program = Program {
        items: vec![Item::Function(FuncDef {
            name: "main".into(),
            params: vec![],
            locals: vec![],
            ret: Type::Int,
            body: Block {
                stmts: vec![
                    Stmt::Expr(Expr::Binary {
                        op: BinOp::Add,
                        left: Box::new(Expr::Num(1)),
                        right: Box::new(Expr::Num(2)),
                    }),
                    Stmt::Return(Some(Expr::Num(5))),
                ],
            },
        })],
    };
    assert_eq!(run_ast(program), 5);
}
