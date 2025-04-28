//! Compiler from AST to bytecode, and Virtual Machine to run it.

use crate::ast::*;
use crate::bytecode::{Instruction, OpCode};

/// Compile a full C4 program into bytecode instructions.
/// Right now, we just compile the "main" function.
/// Later, you can extend it for multiple functions/globals.
pub fn compile(program: &Program) -> Vec<Instruction> {
    let mut code = Vec::new();

    // For now, only compile main function body
    for item in &program.items {
        if let Item::Function(func) = item {
            if func.name == "main" {
                compile_block(&func.body, &mut code);
                code.push(Instruction::new(OpCode::Exit)); // Program exit after main
            }
        }
    }

    code
}

/// Recursively compile a Block.
fn compile_block(block: &Block, code: &mut Vec<Instruction>) {
    for stmt in &block.stmts {
        compile_stmt(stmt, code);
    }
}

/// Compile a statement.
fn compile_stmt(stmt: &Stmt, code: &mut Vec<Instruction>) {
    match stmt {
        Stmt::Expr(expr) => {
            compile_expr(expr, code);
            // After expression statements, discard result if needed
            // No explicit POP needed if design allows
        }
        Stmt::Return(Some(expr)) => {
            compile_expr(expr, code);
            code.push(Instruction::new(OpCode::Exit));
        }
        Stmt::Return(None) => {
            code.push(Instruction::new(OpCode::Exit));
        }
        Stmt::Block(inner) => {
            compile_block(inner, code);
        }
        Stmt::If { cond, then_branch, else_branch } => {
            compile_expr(cond, code);
            code.push(Instruction::with_operand(OpCode::Jz, 0)); // Patch later
            let jz_pos = code.len() - 1;

            compile_stmt(then_branch, code);

            if let Some(else_branch) = else_branch {
                code.push(Instruction::with_operand(OpCode::Jmp, 0)); // Patch later
                let jmp_pos = code.len() - 1;

                let after_then = code.len() as i32;
                code[jz_pos].operand = Some(after_then);

                compile_stmt(else_branch, code);

                let after_else = code.len() as i32;
                code[jmp_pos].operand = Some(after_else);
            } else {
                let after_then = code.len() as i32;
                code[jz_pos].operand = Some(after_then);
            }
        }
        Stmt::While { cond, body } => {
            let loop_start = code.len() as i32;
            compile_expr(cond, code);
            code.push(Instruction::with_operand(OpCode::Jz, 0)); // Patch later
            let jz_pos = code.len() - 1;

            compile_stmt(body, code);

            code.push(Instruction::with_operand(OpCode::Jmp, loop_start));
            let after_loop = code.len() as i32;
            code[jz_pos].operand = Some(after_loop);
        }
        Stmt::Empty => { /* no-op */ }
    }
}

/// Compile an expression and leave the result on the stack.
fn compile_expr(expr: &Expr, code: &mut Vec<Instruction>) {
    match expr {
        Expr::Num(n) => {
            code.push(Instruction::with_operand(OpCode::Imm, *n as i32));
        }
        Expr::Unary { op, expr } => {
            compile_expr(expr, code);
            match op {
                UnOp::Neg => {
                    code.push(Instruction::with_operand(OpCode::Imm, 0));
                    code.push(Instruction::new(OpCode::Sub));
                }
                UnOp::Not => {
                    code.push(Instruction::new(OpCode::Not));
                }
                _ => unimplemented!("Unary operator {:?}", op),
            }
        }
        Expr::Binary { op, left, right } => {
            compile_expr(left, code);
            code.push(Instruction::new(OpCode::Push));
            compile_expr(right, code);

            match op {
                BinOp::Add => code.push(Instruction::new(OpCode::Add)),
                BinOp::Sub => code.push(Instruction::new(OpCode::Sub)),
                BinOp::Mul => code.push(Instruction::new(OpCode::Mul)),
                BinOp::Div => code.push(Instruction::new(OpCode::Div)),
                BinOp::Mod => code.push(Instruction::new(OpCode::Mod)),

                BinOp::Eq => code.push(Instruction::new(OpCode::Eq)),
                BinOp::Ne => code.push(Instruction::new(OpCode::Ne)),
                BinOp::Lt => code.push(Instruction::new(OpCode::Lt)),
                BinOp::Le => code.push(Instruction::new(OpCode::Le)),
                BinOp::Gt => code.push(Instruction::new(OpCode::Gt)),
                BinOp::Ge => code.push(Instruction::new(OpCode::Ge)),

                BinOp::BitAnd => code.push(Instruction::new(OpCode::And)),
                BinOp::BitOr => code.push(Instruction::new(OpCode::Or)),
                BinOp::Xor => code.push(Instruction::new(OpCode::Xor)),
                BinOp::Shl => code.push(Instruction::new(OpCode::Shl)),
                BinOp::Shr => code.push(Instruction::new(OpCode::Shr)),

                _ => unimplemented!("Binary operator {:?}", op),
            }
        }
        Expr::Conditional { cond, then_expr, else_expr } => {
            compile_expr(cond, code);
            code.push(Instruction::with_operand(OpCode::Jz, 0)); // Patch later
            let jz_pos = code.len() - 1;

            compile_expr(then_expr, code);
            code.push(Instruction::with_operand(OpCode::Jmp, 0)); // Patch later
            let jmp_pos = code.len() - 1;

            let after_then = code.len() as i32;
            code[jz_pos].operand = Some(after_then);

            compile_expr(else_expr, code);

            let after_else = code.len() as i32;
            code[jmp_pos].operand = Some(after_else);
        }
        Expr::Var(name) => {
            // Placeholder for future variables
            panic!("Variable lookup not yet implemented: {}", name);
        }
        Expr::Call { callee, args } => {
            // Only allow simple function calls for now
            if let Expr::Var(name) = &**callee {
                if name == "printf" {
                    // Hardcoded printf syscall
                    for arg in args.iter().rev() {
                        compile_expr(arg, code);
                        code.push(Instruction::new(OpCode::Push));
                    }
                    code.push(Instruction::with_operand(OpCode::Printf, args.len() as i32));
                } else {
                    panic!("Only printf supported for now");
                }
            } else {
                panic!("Unsupported callee {:?}", callee);
            }
        }
        _ => unimplemented!("Expression {:?}", expr),
    }
}

/// Virtual machine state.
pub struct VM {
    pub stack: Vec<i32>,
    pub pc: usize,
    pub acc: i32,
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(1024),
            pc: 0,
            acc: 0,
        }
    }

    /// Run a program (bytecode sequence) and return exit code.
    pub fn run(&mut self, code: &[Instruction]) -> i32 {
        while self.pc < code.len() {
            let instr = &code[self.pc];
            self.pc += 1;

            match instr.opcode {
                OpCode::Imm => {
                    self.acc = instr.operand.expect("IMM needs operand");
                }
                OpCode::Push => {
                    self.stack.push(self.acc);
                }
                OpCode::Add => {
                    let val = self.stack.pop().expect("Stack underflow");
                    self.acc += val;
                }
                OpCode::Sub => {
                    let val = self.stack.pop().expect("Stack underflow");
                    self.acc = val - self.acc;
                }
                OpCode::Mul => {
                    let val = self.stack.pop().expect("Stack underflow");
                    self.acc *= val;
                }
                OpCode::Div => {
                    let val = self.stack.pop().expect("Stack underflow");
                    self.acc = val / self.acc;
                }
                OpCode::Mod => {
                    let val = self.stack.pop().expect("Stack underflow");
                    self.acc = val % self.acc;
                }
                OpCode::Eq => {
                    let val = self.stack.pop().expect("Stack underflow");
                    self.acc = (val == self.acc) as i32;
                }
                OpCode::Ne => {
                    let val = self.stack.pop().expect("Stack underflow");
                    self.acc = (val != self.acc) as i32;
                }
                OpCode::Lt => {
                    let val = self.stack.pop().expect("Stack underflow");
                    self.acc = (val < self.acc) as i32;
                }
                OpCode::Le => {
                    let val = self.stack.pop().expect("Stack underflow");
                    self.acc = (val <= self.acc) as i32;
                }
                OpCode::Gt => {
                    let val = self.stack.pop().expect("Stack underflow");
                    self.acc = (val > self.acc) as i32;
                }
                OpCode::Ge => {
                    let val = self.stack.pop().expect("Stack underflow");
                    self.acc = (val >= self.acc) as i32;
                }
                OpCode::And => {
                    let val = self.stack.pop().expect("Stack underflow");
                    self.acc &= val;
                }
                OpCode::Or => {
                    let val = self.stack.pop().expect("Stack underflow");
                    self.acc |= val;
                }
                OpCode::Xor => {
                    let val = self.stack.pop().expect("Stack underflow");
                    self.acc ^= val;
                }
                OpCode::Shl => {
                    let val = self.stack.pop().expect("Stack underflow");
                    self.acc = val << self.acc;
                }
                OpCode::Shr => {
                    let val = self.stack.pop().expect("Stack underflow");
                    self.acc = val >> self.acc;
                }
                OpCode::Not => {
                    self.acc = !self.acc;
                }
                OpCode::Jmp => {
                    self.pc = instr.operand.expect("JMP needs target") as usize;
                }
                OpCode::Jz => {
                    if self.acc == 0 {
                        self.pc = instr.operand.expect("JZ needs target") as usize;
                    }
                }
                OpCode::Jnz => {
                    if self.acc != 0 {
                        self.pc = instr.operand.expect("JNZ needs target") as usize;
                    }
                }
                OpCode::Exit => {
                    return self.acc;
                }
                OpCode::Printf => {
                    println!("(printf stub) {}", self.acc);
                }
                _ => {
                    panic!("Unsupported opcode: {:?}", instr.opcode);
                }
            }
        }

        self.acc
    }
}

