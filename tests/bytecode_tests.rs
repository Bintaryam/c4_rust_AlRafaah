use c4_rust_alrafaah::bytecode::{Instruction, OpCode};

#[test]
fn test_instruction_no_operand() {
    let instr = Instruction::new(OpCode::Add);
    assert_eq!(instr.opcode, OpCode::Add);
    assert!(instr.operand.is_none());
}

#[test]
fn test_instruction_with_operand() {
    let instr = Instruction::with_operand(OpCode::Imm, 123);
    assert_eq!(instr.opcode, OpCode::Imm);
    assert_eq!(instr.operand, Some(123));
}

#[test]
fn test_opcode_enum() {
    // Just confirming discriminants are stable & match assumptions
    assert_eq!(OpCode::Imm as u8, 0);
    assert_eq!(OpCode::Push as u8, 5);
}

