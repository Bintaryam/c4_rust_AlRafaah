use c4_rust_AlRafaah::bytecode::*;
use c4_rust_AlRafaah::vm::VM;

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
    chunk.push(OpCode::LT); // 10 < 20 = 1
    chunk.push(OpCode::EXIT);
    assert_eq!(run_chunk(chunk), 1);
}

#[test]
fn test_conditional_jump_false() {
    let mut chunk = Chunk::default();
    chunk.push_int(OpCode::IMM, 0);
    chunk.push_jump(OpCode::BZ, 4);
    chunk.push_int(OpCode::IMM, 100); // skipped
    chunk.push(OpCode::JMP);
    chunk.push_int(OpCode::IMM, 42); // target
    chunk.push(OpCode::EXIT);

    assert_eq!(run_chunk(chunk), 42);
}

#[test]
fn test_conditional_jump_true() {
    let mut chunk = Chunk::default();
    chunk.push_int(OpCode::IMM, 1);
    chunk.push_jump(OpCode::BZ, 4);
    chunk.push_int(OpCode::IMM, 42); // not skipped
    chunk.push(OpCode::EXIT);
    chunk.push_int(OpCode::IMM, 999); // would be skipped

    assert_eq!(run_chunk(chunk), 42);
}

#[test]
fn test_stack_and_load_store() {
    let mut chunk = Chunk::default();

    // Function body (will be appended after main)
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
    let entry_point = wrapper.code.len() + 2; // main + exit = 2
    wrapper.push_call(OpCode::JSR, entry_point);
    wrapper.push(OpCode::EXIT);
    wrapper.code.extend(body.code);

    assert_eq!(run_chunk(wrapper), 123);
}



#[test]
fn test_nested_arithmetic() {
    let mut chunk = Chunk::default();
    // (5 + 3) * 2 = 16
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
    chunk.push(OpCode::EQ); // 10 == 10 -> 1
    chunk.push(OpCode::EXIT);

    assert_eq!(run_chunk(chunk), 1);
}

#[test]
fn test_call_and_return() {
    let mut chunk = Chunk::default();

    // main
    chunk.push_call(OpCode::JSR, 2); // skip main
    chunk.push(OpCode::EXIT);       // 1

    // function body
    chunk.push_int(OpCode::IMM, 42); // 2
    chunk.push(OpCode::LEV);         // 3

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


