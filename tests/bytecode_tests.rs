// tests/bytecode_tests.rs

use c4_rust_AlRafaah::bytecode::*;

#[test]
fn test_push_basic_opcodes() {
    let mut chunk = Chunk::default();
    chunk.push(OpCode::ADD);
    chunk.push(OpCode::SUB);
    chunk.push(OpCode::MUL);

    assert_eq!(chunk.code.len(), 3);
    assert_eq!(chunk.code[0], Instruction::Instr(OpCode::ADD));
    assert_eq!(chunk.code[1], Instruction::Instr(OpCode::SUB));
    assert_eq!(chunk.code[2], Instruction::Instr(OpCode::MUL));
}

#[test]
fn test_push_immediate_values() {
    let mut chunk = Chunk::default();
    chunk.push_int(OpCode::IMM, 42);
    chunk.push_int(OpCode::IMM, -1);
    chunk.push_int(OpCode::LEA, 16);

    assert_eq!(chunk.code.len(), 3);
    assert_eq!(chunk.code[0], Instruction::InstrInt(OpCode::IMM, 42));
    assert_eq!(chunk.code[1], Instruction::InstrInt(OpCode::IMM, -1));
    assert_eq!(chunk.code[2], Instruction::InstrInt(OpCode::LEA, 16));
}

#[test]
fn test_jump_and_call_instructions() {
    let mut chunk = Chunk::default();
    chunk.push_jump(OpCode::JMP, 10);
    chunk.push_jump(OpCode::BZ, 20);
    chunk.push_call(OpCode::JSR, 30);

    assert_eq!(chunk.code.len(), 3);
    assert_eq!(chunk.code[0], Instruction::Jump(OpCode::JMP, 10));
    assert_eq!(chunk.code[1], Instruction::Jump(OpCode::BZ, 20));
    assert_eq!(chunk.code[2], Instruction::Call(OpCode::JSR, 30));
}

#[test]
fn test_instruction_order_preservation() {
    let mut chunk = Chunk::default();
    chunk.push(OpCode::PSH);
    chunk.push_int(OpCode::IMM, 99);
    chunk.push(OpCode::ADD);

    assert_eq!(chunk.code[0], Instruction::Instr(OpCode::PSH));
    assert_eq!(chunk.code[1], Instruction::InstrInt(OpCode::IMM, 99));
    assert_eq!(chunk.code[2], Instruction::Instr(OpCode::ADD));
}

#[test]
fn test_multiple_opcodes_comprehensively() {
    let mut chunk = Chunk::default();

    let opcodes = vec![
        OpCode::IMM, OpCode::LI, OpCode::LC,
        OpCode::SI, OpCode::SC, OpCode::PSH,
        OpCode::OR, OpCode::XOR, OpCode::AND,
        OpCode::EQ, OpCode::NE, OpCode::LT,
        OpCode::LE, OpCode::GT, OpCode::GE,
        OpCode::SHL, OpCode::SHR, OpCode::ADD,
        OpCode::SUB, OpCode::MUL, OpCode::DIV, OpCode::MOD
    ];

    for op in &opcodes {
        chunk.push(*op);
    }

    assert_eq!(chunk.code.len(), opcodes.len());

    for (i, op) in opcodes.iter().enumerate() {
        assert_eq!(chunk.code[i], Instruction::Instr(*op));
    }
}

#[test]
fn test_syscall_opcodes() {
    let mut chunk = Chunk::default();
    let syscalls = vec![
        OpCode::OPEN, OpCode::READ, OpCode::CLOS,
        OpCode::PRTF, OpCode::MALC, OpCode::FREE,
        OpCode::MSET, OpCode::MCMP, OpCode::EXIT,
    ];

    for op in &syscalls {
        chunk.push(*op);
    }

    for (i, op) in syscalls.iter().enumerate() {
        assert_eq!(chunk.code[i], Instruction::Instr(*op));
    }
}
