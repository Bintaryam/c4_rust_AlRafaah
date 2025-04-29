/// Defines the bytecode instruction set, along with helper data structures
/// that represent compiled bytecode chunks in the Rust version of the C4 compiler.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OpCode {
    // Stack/Memory
    LEA,
    IMM,
    JMP,
    JSR,
    BZ,
    BNZ,
    ENT,
    ADJ,
    LEV,
    LI,
    LC,
    SI,
    SC,
    PSH,

    // Binary operations
    OR,
    XOR,
    AND,
    EQ,
    NE,
    LT,
    GT,
    LE,
    GE,
    SHL,
    SHR,
    ADD,
    SUB,
    MUL,
    DIV,
    MOD,

    // System calls (as seen in original c4)
    OPEN,
    READ,
    CLOS,
    PRTF,
    MALC,
    FREE,
    MSET,
    MCMP,
    EXIT,
}

/// A single instruction can be an OpCode with optional operands
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    /// Raw instruction with optional operands
    Instr(OpCode),
    /// Instruction with immediate value (e.g. IMM, LEA, ADJ)
    InstrInt(OpCode, i64),
    /// Jump instructions with target address
    Jump(OpCode, usize),
    /// Call instruction with target address
    Call(OpCode, usize),
}

/// Represents a compiled chunk of instructions
#[derive(Debug, Default)]
pub struct Chunk {
    pub code: Vec<Instruction>,
}

impl Chunk {
    /// Add a no-operand instruction
    pub fn push(&mut self, op: OpCode) {
        self.code.push(Instruction::Instr(op));
    }

    /// Add an instruction with an integer operand (e.g., IMM 42)
    pub fn push_int(&mut self, op: OpCode, val: i64) {
        self.code.push(Instruction::InstrInt(op, val));
    }

    /// Add a jump instruction
    pub fn push_jump(&mut self, op: OpCode, target: usize) {
        self.code.push(Instruction::Jump(op, target));
    }

    /// Add a call instruction
    pub fn push_call(&mut self, op: OpCode, target: usize) {
        self.code.push(Instruction::Call(op, target));
    }

    /// Debug helper to print all instructions
    pub fn dump(&self) {
        for (i, instr) in self.code.iter().enumerate() {
            println!("{:04}: {:?}", i, instr);
        }
    }
}
