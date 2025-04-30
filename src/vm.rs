use crate::bytecode::{Chunk, Instruction, OpCode};

pub struct VM {
    stack: Vec<i64>,
    call_stack: Vec<(usize, usize, usize)>, // (return_pc, old_bp, old_fp)
    pc: usize,
    sp: usize,
    bp: usize, // current stack top (unused now)
    fp: usize, // base of current function frame (for LEA)
    pub debug: bool,
}

impl VM {
    pub fn new() -> Self {
        VM {
            stack: vec![0; 1024 * 1024],
            call_stack: Vec::new(),
            pc: 0,
            sp: 0,
            bp: 0,
            fp: 0,
            debug: false,
        }
    }

    pub fn run(&mut self, chunk: &Chunk) -> i64 {
        let code = &chunk.code;
        let mut a: i64 = 0;

        while self.pc < code.len() {
            let instr = &code[self.pc];
            self.pc += 1;

            if self.debug {
                println!("{:04} {:?}", self.pc - 1, instr);
            }

            match instr {
                Instruction::Instr(op) => match op {
                    OpCode::ADD => a = self.pop() + a,
                    OpCode::SUB => a = self.pop() - a,
                    OpCode::MUL => a = self.pop() * a,
                    OpCode::DIV => a = self.pop() / a,
                    OpCode::MOD => a = self.pop() % a,

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

                    OpCode::PSH => self.push(a),

                    OpCode::LEV => {
                        let ret_val = a;
                        let (ret_pc, old_sp, old_fp) = self.call_stack.pop().expect("call stack underflow");
                        self.pc = ret_pc;
                        self.sp = old_sp;
                        self.fp = old_fp;
                        a = ret_val;
                    }

                    OpCode::EXIT => {
                        println!("exit({a})");
                        return a;
                    }

                    _ => unimplemented!("{:?}", op),
                },

                Instruction::InstrInt(op, val) => match op {
                    OpCode::IMM => a = *val,
                    OpCode::LEA => a = (self.fp + *val as usize) as i64,
                    OpCode::ADJ => {
                        for _ in 0..*val {
                            self.pop();
                        }
                    }
                    OpCode::ENT => {
                        self.call_stack.push((self.pc, self.sp, self.fp));
                        self.fp = self.sp;
                        for _ in 0..*val {
                            self.push(0);
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

    fn push(&mut self, val: i64) {
        if self.sp >= self.stack.len() {
            panic!("stack overflow");
        }
        self.stack[self.sp] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> i64 {
        if self.sp == 0 {
            panic!("stack underflow");
        }
        self.sp -= 1;
        self.stack[self.sp]
    }
}
