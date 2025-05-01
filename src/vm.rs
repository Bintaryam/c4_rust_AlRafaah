// Import definitions for bytecode instructions, AST nodes, etc.
use crate::bytecode::{Chunk, Instruction, OpCode};
use crate::ast::*;

// Compile a full program by compiling each item (e.g., function) into the bytecode chunk.
impl Program {
    pub fn compile(&self, chunk: &mut Chunk) -> Result<(), String> {
        for item in &self.items {
            item.compile(chunk)?;
        }
        Ok(())
    }
}

// Compile an individual top-level item. Currently only functions are handled.
impl Item {
    pub fn compile(&self, chunk: &mut Chunk) -> Result<(), String> {
        match self {
            Item::Function(f) => f.compile(chunk),
            _ => Ok(()), // Global variables or enums are not compiled yet.
        }
    }
}

// Compile a function definition into bytecode.
impl FuncDef {
    pub fn compile(&self, chunk: &mut Chunk) -> Result<(), String> {
        if self.name == "main" {
            // Special handling for `main` as the program entry point.
            let entry = chunk.code.len() + 2; // Location where function starts.
            chunk.push_call(OpCode::JSR, entry); // Insert jump to subroutine.
            chunk.push(OpCode::EXIT); // Exit program after `main` returns.
        }

        // Reserve space for local variables.
        let local_count = self.locals.len() as i64;
        chunk.push_int(OpCode::ENT, local_count); // Enter function frame.

        // Compile each statement in the function body.
        for stmt in &self.body.stmts {
            stmt.compile(chunk)?;
        }

        // Leave function.
        chunk.push(OpCode::LEV);
        Ok(())
    }
}

// Compile statements to bytecode.
impl Stmt {
    pub fn compile(&self, chunk: &mut Chunk) -> Result<(), String> {
        match self {
            Stmt::Expr(e) => {
                e.compile(chunk)?;
                Ok(()) // Expression result left in register `a`.
            }
            Stmt::Return(Some(e)) => {
                e.compile(chunk)?;
                chunk.push(OpCode::LEV); // Return from function.
                Ok(())
            }
            Stmt::Return(None) => {
                chunk.push(OpCode::LEV);
                Ok(())
            }
            Stmt::Block(b) => {
                for stmt in &b.stmts {
                    stmt.compile(chunk)?;
                }
                Ok(())
            }
            _ => Ok(()), // Other statement types not yet implemented.
        }
    }
}

// Compile expressions into bytecode.
impl Expr {
    pub fn compile(&self, chunk: &mut Chunk) -> Result<(), String> {
        match self {
            Expr::Num(n) => chunk.push_int(OpCode::IMM, *n), // Load immediate integer.
            Expr::Binary { op, left, right } => {
                left.compile(chunk)?;
                chunk.push(OpCode::PSH); // Save left operand.
                right.compile(chunk)?;

                // Map binary operation to opcode.
                let code = match op {
                    BinOp::Add => OpCode::ADD,
                    BinOp::Sub => OpCode::SUB,
                    BinOp::Mul => OpCode::MUL,
                    BinOp::Div => OpCode::DIV,
                    BinOp::Mod => OpCode::MOD,
                    BinOp::Eq  => OpCode::EQ,
                    BinOp::Ne  => OpCode::NE,
                    BinOp::Lt  => OpCode::LT,
                    BinOp::Le  => OpCode::LE,
                    BinOp::Gt  => OpCode::GT,
                    BinOp::Ge  => OpCode::GE,
                    BinOp::BitAnd => OpCode::AND,
                    BinOp::BitOr  => OpCode::OR,
                    BinOp::Xor    => OpCode::XOR,
                    BinOp::Shl    => OpCode::SHL,
                    BinOp::Shr    => OpCode::SHR,
                    _ => return Err(format!("unsupported op: {:?}", op)),
                };

                chunk.push(code);
            }
            Expr::Call { callee, args } => {
                for arg in args {
                    arg.compile(chunk)?;
                    chunk.push(OpCode::PSH); // Push each argument.
                }

                // Handle only direct calls to named functions for now.
                if let Expr::Var(name) = &**callee {
                    if name == "main" {
                        chunk.push_call(OpCode::JSR, 2); // Hardcoded address for `main`.
                    } else {
                        return Err(format!("unsupported function call: {}", name));
                    }
                } else {
                    return Err("callee must be a named function".into());
                }
            }
            _ => return Err(format!("unsupported expr: {:?}", self)),
        }
        Ok(())
    }
}

// Virtual Machine structure.
pub struct VM {
    stack: Vec<i64>,                        // Operand stack.
    call_stack: Vec<(usize, usize, usize)>, // Stores (return_pc, old_sp, old_fp).
    pc: usize,                              // Program counter.
    sp: usize,                              // Stack pointer.
    bp: usize,                              // Base pointer (currently unused).
    fp: usize,                              // Frame pointer for current function call.
    pub debug: bool,                        // Debug flag.
}

impl VM {
    // Constructor: Initialize VM with preallocated stack.
    pub fn new() -> Self {
        VM {
            stack: vec![0; 1024 * 1024], // 1 MB stack space.
            call_stack: Vec::new(),
            pc: 0,
            sp: 0,
            bp: 0,
            fp: 0,
            debug: false,
        }
    }

    // Execute bytecode in a given chunk.
    pub fn run(&mut self, chunk: &Chunk) -> i64 {
        let code = &chunk.code;
        let mut a: i64 = 0; // Register `a` is used for computation.

        while self.pc < code.len() {
            let instr = &code[self.pc];
            self.pc += 1;

            if self.debug {
                println!("{:04} {:?}", self.pc - 1, instr);
            }

            match instr {
                Instruction::Instr(op) => match op {
                    // Arithmetic
                    OpCode::ADD => a = self.pop() + a,
                    OpCode::SUB => a = self.pop() - a,
                    OpCode::MUL => a = self.pop() * a,
                    OpCode::DIV => a = self.pop() / a,
                    OpCode::MOD => a = self.pop() % a,

                    // Bitwise and comparison
                    OpCode::AND => a = self.pop() & a,
                    OpCode::OR => a = self.pop() | a,
                    OpCode::XOR => a = self.pop() ^ a,
                    OpCode::EQ => a = (self.pop() == a) as i64,
                    OpCode::NE => a = (self.pop() != a) as i64,
                    OpCode::LT => a = (self.pop() < a) as i64,
                    OpCode::LE => a = (self.pop() <= a) as i64,
                    OpCode::GT => a = (self.pop() > a) as i64,
                    OpCode::GE => a = (self.pop() >= a) as i64,
                    OpCode::SHL => a = self.pop() << a,
                    OpCode::SHR => a = self.pop() >> a,

                    // Memory access
                    OpCode::LI => a = self.stack[a as usize],
                    OpCode::LC => a = self.stack[a as usize] & 0xFF,
                    OpCode::SI => {
                        let addr = self.pop() as usize;
                        self.stack[addr] = a;
                        a = self.stack[addr];
                    }
                    OpCode::SC => {
                        let addr = self.pop() as usize;
                        self.stack[addr] = a & 0xFF;
                        a = self.stack[addr];
                    }

                    OpCode::PSH => self.push(a), // Push register `a` onto stack.

                    // Function return
                    OpCode::LEV => {
                        let ret_val = a;
                        let (ret_pc, old_sp, old_fp) = self.call_stack.pop().expect("call stack underflow");
                        self.pc = ret_pc;
                        self.sp = old_sp;
                        self.fp = old_fp;
                        a = ret_val;
                    }

                    // Exit program
                    OpCode::EXIT => {
                        println!("exit({a})");
                        return a;
                    }

                    _ => unimplemented!("{:?}", op),
                },

                Instruction::InstrInt(op, val) => match op {
                    OpCode::IMM => a = *val,                            // Load immediate value.
                    OpCode::LEA => a = (self.fp + *val as usize) as i64, // Compute effective address.
                    OpCode::ADJ => {
                        for _ in 0..*val {
                            self.pop(); // Discard arguments.
                        }
                    }
                    OpCode::ENT => {
                        // Enter function call.
                        self.call_stack.push((self.pc, self.sp, self.fp));
                        self.fp = self.sp;
                        for _ in 0..*val {
                            self.push(0); // Allocate local variables.
                        }
                    }
                    _ => panic!("Unhandled: {:?}", op),
                },

                Instruction::Jump(op, target) => match op {
                    OpCode::JMP => self.pc = *target,
                    OpCode::BZ => if a == 0 { self.pc = *target; },
                    OpCode::BNZ => if a != 0 { self.pc = *target; },
                    _ => panic!("Invalid jump: {:?}", op),
                },

                Instruction::Call(op, target) => match op {
                    OpCode::JSR => {
                        self.call_stack.push((self.pc, self.sp, self.fp));
                        self.pc = *target;
                    }
                    _ => panic!("Invalid call: {:?}", op),
                },
            }
        }

        a
    }

    // Push value to stack.
    fn push(&mut self, val: i64) {
        if self.sp >= self.stack.len() {
            panic!("stack overflow");
        }
        self.stack[self.sp] = val;
        self.sp += 1;
    }

    // Pop value from stack.
    fn pop(&mut self) -> i64 {
        if self.sp == 0 {
            panic!("stack underflow");
        }
        self.sp -= 1;
        self.stack[self.sp]
    }
}

