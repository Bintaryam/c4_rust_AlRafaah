//! Bytecode instruction set for the C4 virtual machine.

/// Virtual machine opcodes (operations).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OpCode {
    // Stack and memory operations
    Imm,    // Load immediate value into accumulator
    Lc,     // Load char from address in accumulator
    Li,     // Load int from address in accumulator
    Sc,     // Store char to address on stack
    Si,     // Store int to address on stack
    Push,   // Push accumulator onto stack
    Pop,    // Pop stack (optional if needed)

    // Arithmetic operations
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // Logical and bitwise operations
    Shl,
    Shr,
    And,
    Or,
    Xor,
    Not,    // Bitwise NOT (~)

    // Comparison operations (results in 0 or 1)
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // Control flow
    Jmp,    // Unconditional jump
    Jz,     // Jump if accumulator == 0
    Jnz,    // Jump if accumulator != 0
    Call,   // Call subroutine
    Ent,    // Enter subroutine: setup stack frame
    Adj,    // Adjust stack pointer after call
    Lev,    // Leave subroutine: teardown stack frame
    Ret,    // Return from subroutine (alias for Lev or special)

    // System calls / runtime
    Exit,   // Exit program
    Open,
    Read,
    Close,
    Printf,
    Malloc,
    Free,
    Memset,
    Memcpy,

    // Additional
    Halt,   // Stop execution (for safety)
}

/// A single bytecode instruction.
/// Some opcodes have an optional operand (e.g., IMM 42).
#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: OpCode,
    pub operand: Option<i32>,
}

impl Instruction {
    /// Helper to create an instruction without an operand.
    pub fn new(opcode: OpCode) -> Self {
        Self {
            opcode,
            operand: None,
        }
    }

    /// Helper to create an instruction with an operand.
    pub fn with_operand(opcode: OpCode, operand: i32) -> Self {
        Self {
            opcode,
            operand: Some(operand),
        }
    }
}

