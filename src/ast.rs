// src/ast.rs

//! Abstract Syntax Tree (AST) for the C4 compiler subset in Rust.

/// A full C4 program: a list of top-level items.
#[derive(Debug, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
}

/// Top-level items: global variables, functions, or enum declarations.
#[derive(Debug, PartialEq)]
pub enum Item {
    Global(GlobalDecl),
    Function(FuncDef),
    Enum(EnumDecl),
}

/// A global variable declaration: e.g., `int x;` or `char *p;`
#[derive(Debug, PartialEq)]
pub struct GlobalDecl {
    /// Name and type of a single declarator.  
    /// (Comma‐separated lists of globals are emitted as multiple `GlobalDecl` items.)
    pub name: String,
    pub ty: Type,
}

/// An anonymous enum declaration: e.g., `enum { A = 0, B, C = 5 };`
#[derive(Debug, PartialEq)]
pub struct EnumDecl {
    /// List of (name, optional initializer)
    pub variants: Vec<(String, Option<i64>)>,
}

/// A function definition: `int f(int a, char b) { ... }`
#[derive(Debug, PartialEq)]
pub struct FuncDef {
    pub ret: Type,
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub locals: Vec<(String, Type)>,
    pub body: Block,
}


/// A block `{ ... }`: a sequence of statements.
#[derive(Debug, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

/// Statements in C4.
#[derive(Debug, PartialEq)]
pub enum Stmt {
    If {
        cond: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        cond: Expr,
        body: Box<Stmt>,
    },
    Return(Option<Expr>),
    Expr(Expr),   // expression statement `expr;`
    Block(Block), // nested block
    Empty,        // empty statement `;`
}

/// Expressions in C4.
#[derive(Debug, PartialEq)]
pub enum Expr {
    Num(i64),
    Str(String),
    Var(String),
    Unary {
        op: UnOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Cast {
        ty: Type,
        expr: Box<Expr>,
    },
    SizeOf(Type),
    Conditional {
        cond: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
    },
    /// Array indexing `array[index]`
    Index {
        array: Box<Expr>,
        index: Box<Expr>,
    },
}

/// Binary operators in C4.
#[derive(Debug, PartialEq)]
pub enum BinOp {
    Assign,        // =
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Le, Gt, Ge,
    BitAnd,        // &
    BitOr,         // |
    Xor,           // ^
    Shl, Shr,      // << >>
    LogAnd,        // &&
    LogOr,         // ||
}

/// Unary operators, including prefix/postfix.
#[derive(Debug, PartialEq)]
pub enum UnOp {
    PreInc,   // ++x
    PreDec,   // --x
    PostInc,  // x++
    PostDec,  // x--
    Plus,     // +x
    Neg,      // -x
    Not,      // !x
    BitNot,   // ~x
    Deref,    // *x
    Addr,     // &x
}

/// Types in C4: void, int, char, or pointer to.
#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Void,
    Int,
    Char,
    Ptr(Box<Type>),
}
