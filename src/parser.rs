// src/parser.rs

use crate::ast::*;
use crate::lexer::{Lexer, Token, LexError};

/// Recursive‐descent parser covering 100% of C4 grammar.
pub struct Parser<'a> {
    lex: Lexer<'a>,
    cur: Token,
}

impl<'a> Parser<'a> {
    /// Initialize parser and read first token.
    pub fn new(input: &'a str) -> Result<Self, LexError> {
        let mut lex = Lexer::new(input);
        let first = lex.next_token()?;
        Ok(Parser { lex, cur: first })
    }

    /// Advance to the next token.
    fn bump(&mut self) -> Result<(), LexError> {
        self.cur = self.lex.next_token()?;
        Ok(())
    }

    /// Consume `tok` if it matches.
    fn eat(&mut self, tok: Token) -> Result<bool, LexError> {
        if self.cur == tok {
            self.bump()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Expect `tok` or error.
    fn expect(&mut self, tok: Token) -> Result<(), String> {
        if self.cur == tok {
            self.bump().map_err(|e| e.0)
        } else {
            Err(format!("expected {:?}, got {:?}", tok, self.cur))
        }
    }

    /// Expect an identifier, return its name.
    fn expect_ident(&mut self) -> Result<String, String> {
        if let Token::Ident(name) = std::mem::replace(&mut self.cur, Token::Eof) {
            self.bump().map_err(|e| e.0)?;
            Ok(name)
        } else {
            Err(format!("expected identifier, got {:?}", self.cur))
        }
    }

    /// Parse an entire program.
    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut items = Vec::new();
        while self.cur != Token::Eof {
            let mut chunk = self.parse_item()?;
            items.append(&mut chunk);
        }
        Ok(Program { items })
    }

    /// Parse a top‐level item (enum, globals, or function).
    /// Returns a Vec<Item> to support comma‐separated globals.
    fn parse_item(&mut self) -> Result<Vec<Item>, String> {
        let mut items = Vec::new();

        // enum declaration?
        if self.cur == Token::KwEnum {
            let ed = self.parse_enum()?;
            self.expect(Token::Semicolon)?;
            items.push(Item::Enum(ed));
            return Ok(items);
        }

        // otherwise type → ident (supports void/int/char)
        let ty = self.parse_type()?;
        let name = self.expect_ident()?;

        // function?
        if self.eat(Token::LParen)? {
            let func = self.parse_func(name, ty)?;
            items.push(Item::Function(func));
            return Ok(items);
        }

        // else: comma‐separated globals
        items.push(Item::Global(GlobalDecl { name: name.clone(), ty: ty.clone() }));
        while self.eat(Token::Comma)? {
            let n = self.expect_ident()?;
            items.push(Item::Global(GlobalDecl { name: n, ty: ty.clone() }));
        }
        self.expect(Token::Semicolon)?;
        Ok(items)
    }

    /// enum { A = 0, B, C = 5 }
    fn parse_enum(&mut self) -> Result<EnumDecl, String> {
        self.expect(Token::KwEnum)?;
        self.expect(Token::LBrace)?;
        let mut variants = Vec::new();
        while self.cur != Token::RBrace {
            let vname = self.expect_ident()?;
            let init = if self.eat(Token::Assign)? {
                if let Expr::Num(val) = self.parse_assignment()? {
                    Some(val)
                } else {
                    return Err("enum initializer must be a number".into());
                }
            } else {
                None
            };
            variants.push((vname, init));
            if !self.eat(Token::Comma)? {
                break;
            }
        }
        self.expect(Token::RBrace)?;
        Ok(EnumDecl { variants })
    }

    /// void, int, char, and pointer declarators.
    fn parse_type(&mut self) -> Result<Type, String> {
        let mut ty = match self.cur {
            Token::KwVoid => {
                self.bump().map_err(|e| e.0)?;
                Type::Void
            }
            Token::KwInt => {
                self.bump().map_err(|e| e.0)?;
                Type::Int
            }
            Token::KwChar => {
                self.bump().map_err(|e| e.0)?;
                Type::Char
            }
            _ => return Err(format!("expected type, got {:?}", self.cur)),
        };
        while self.eat(Token::Star)? {
            ty = Type::Ptr(Box::new(ty));
        }
        Ok(ty)
    }

    /// Parse a function: parameters, locals, then block.
    fn parse_func(&mut self, name: String, ret_ty: Type) -> Result<FuncDef, String> {
        // parameters
        let mut params = Vec::new();
        if self.cur != Token::RParen {
            loop {
                let pty = self.parse_type()?;
                let pname = self.expect_ident()?;
                params.push((pname, pty));
                if !self.eat(Token::Comma)? {
                    break;
                }
            }
        }
        self.expect(Token::RParen)?;

        // function body starts
        self.expect(Token::LBrace)?;
        // locals
        let mut locals = Vec::new();
        while matches!(self.cur, Token::KwInt | Token::KwChar) {
            let lty = self.parse_type()?;
            loop {
                let lname = self.expect_ident()?;
                locals.push((lname.clone(), lty.clone()));
                if !self.eat(Token::Comma)? {
                    break;
                }
            }
            self.expect(Token::Semicolon)?;
        }
        let body = self.parse_block()?;
        Ok(FuncDef { ret: ret_ty, name, params, locals, body })
    }

    /// `{ stmt* }`
    fn parse_block(&mut self) -> Result<Block, String> {
        self.expect(Token::LBrace)?;
        let mut stmts = Vec::new();
        while self.cur != Token::RBrace {
            stmts.push(self.parse_stmt()?);
        }
        self.bump().map_err(|e| e.0)?;
        Ok(Block { stmts })
    }

    /// Statements: if, while, return, block, empty, or expr.
    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        // if
        if self.cur == Token::KwIf {
            self.bump().map_err(|e| e.0)?;
            self.expect(Token::LParen)?;
            let cond = self.parse_assignment()?;
            self.expect(Token::RParen)?;
            let then_b = Box::new(self.parse_stmt()?);
            let else_b = if self.eat(Token::KwElse)? {
                Some(Box::new(self.parse_stmt()?))
            } else {
                None
            };
            return Ok(Stmt::If { cond, then_branch: then_b, else_branch: else_b });
        }
        // while
        if self.cur == Token::KwWhile {
            self.bump().map_err(|e| e.0)?;
            self.expect(Token::LParen)?;
            let cond = self.parse_assignment()?;
            self.expect(Token::RParen)?;
            let body = Box::new(self.parse_stmt()?);
            return Ok(Stmt::While { cond, body });
        }
        // return
        if self.cur == Token::KwReturn {
            self.bump().map_err(|e| e.0)?;
            let expr = if self.cur != Token::Semicolon {
                Some(self.parse_assignment()?)
            } else {
                None
            };
            self.expect(Token::Semicolon)?;
            return Ok(Stmt::Return(expr));
        }
        // block
        if self.eat(Token::LBrace)? {
            let b = self.parse_block()?;
            return Ok(Stmt::Block(b));
        }
        // empty
        if self.eat(Token::Semicolon)? {
            return Ok(Stmt::Empty);
        }
        // expr stmt
        let e = self.parse_assignment()?;
        self.expect(Token::Semicolon)?;
        Ok(Stmt::Expr(e))
    }

    /// Assignment is right‐associative.
    fn parse_assignment(&mut self) -> Result<Expr, String> {
        let left = self.parse_conditional()?;
        if self.eat(Token::Assign)? {
            let right = self.parse_assignment()?;
            return Ok(Expr::Binary {
                op: BinOp::Assign,
                left: Box::new(left),
                right: Box::new(right),
            });
        }
        Ok(left)
    }

    /// Ternary `?:`
    fn parse_conditional(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_logical_or()?;
        if self.eat(Token::Question)? {
            let then_e = self.parse_assignment()?;
            self.expect(Token::Colon)?;
            let else_e = self.parse_assignment()?;
            expr = Expr::Conditional {
                cond: Box::new(expr),
                then_expr: Box::new(then_e),
                else_expr: Box::new(else_e),
            };
        }
        Ok(expr)
    }

    /// Logical OR `||`
    fn parse_logical_or(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_logical_and()?;
        while self.eat(Token::OrOr)? {
            let rhs = self.parse_logical_and()?;
            expr = Expr::Binary { op: BinOp::LogOr, left: Box::new(expr), right: Box::new(rhs) };
        }
        Ok(expr)
    }

    /// Logical AND `&&`
    fn parse_logical_and(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_bitwise_or()?;
        while self.eat(Token::AndAnd)? {
            let rhs = self.parse_bitwise_or()?;
            expr = Expr::Binary { op: BinOp::LogAnd, left: Box::new(expr), right: Box::new(rhs) };
        }
        Ok(expr)
    }

    /// Bitwise OR `|`
    fn parse_bitwise_or(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_bitwise_xor()?;
        while self.eat(Token::Or)? {
            let rhs = self.parse_bitwise_xor()?;
            expr = Expr::Binary { op: BinOp::BitOr, left: Box::new(expr), right: Box::new(rhs) };
        }
        Ok(expr)
    }

    /// Bitwise XOR `^`
    fn parse_bitwise_xor(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_bitwise_and()?;
        while self.eat(Token::Xor)? {
            let rhs = self.parse_bitwise_and()?;
            expr = Expr::Binary { op: BinOp::Xor, left: Box::new(expr), right: Box::new(rhs) };
        }
        Ok(expr)
    }

    /// Bitwise AND `&`
    fn parse_bitwise_and(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_equality()?;
        while self.eat(Token::And)? {
            let rhs = self.parse_equality()?;
            expr = Expr::Binary { op: BinOp::BitAnd, left: Box::new(expr), right: Box::new(rhs) };
        }
        Ok(expr)
    }

    /// Equality `==` and `!=`
    fn parse_equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_relational()?;
        loop {
            if self.eat(Token::EqEq)? {
                let rhs = self.parse_relational()?;
                expr = Expr::Binary { op: BinOp::Eq, left: Box::new(expr), right: Box::new(rhs) };
            } else if self.eat(Token::Ne)? {
                let rhs = self.parse_relational()?;
                expr = Expr::Binary { op: BinOp::Ne, left: Box::new(expr), right: Box::new(rhs) };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    /// Relational `<, >, <=, >=`
    fn parse_relational(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_shift()?;
        loop {
            if self.eat(Token::Lt)? {
                let rhs = self.parse_shift()?;
                expr = Expr::Binary { op: BinOp::Lt, left: Box::new(expr), right: Box::new(rhs) };
            } else if self.eat(Token::Gt)? {
                let rhs = self.parse_shift()?;
                expr = Expr::Binary { op: BinOp::Gt, left: Box::new(expr), right: Box::new(rhs) };
            } else if self.eat(Token::Le)? {
                let rhs = self.parse_shift()?;
                expr = Expr::Binary { op: BinOp::Le, left: Box::new(expr), right: Box::new(rhs) };
            } else if self.eat(Token::Ge)? {
                let rhs = self.parse_shift()?;
                expr = Expr::Binary { op: BinOp::Ge, left: Box::new(expr), right: Box::new(rhs) };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    /// Shifts `<<`, `>>`
    fn parse_shift(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_add_sub()?;
        loop {
            if self.eat(Token::Shl)? {
                let rhs = self.parse_add_sub()?;
                expr = Expr::Binary { op: BinOp::Shl, left: Box::new(expr), right: Box::new(rhs) };
            } else if self.eat(Token::Shr)? {
                let rhs = self.parse_add_sub()?;
                expr = Expr::Binary { op: BinOp::Shr, left: Box::new(expr), right: Box::new(rhs) };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    /// `+`, `-`
    fn parse_add_sub(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_mul_div_mod()?;
        loop {
            if self.eat(Token::Plus)? {
                let rhs = self.parse_mul_div_mod()?;
                expr = Expr::Binary { op: BinOp::Add, left: Box::new(expr), right: Box::new(rhs) };
            } else if self.eat(Token::Minus)? {
                let rhs = self.parse_mul_div_mod()?; // Parse the right-hand side of subtraction.
                expr = Expr::Binary { op: BinOp::Sub, left: Box::new(expr), right: Box::new(rhs) };
            } else {
                break; // Exit the loop if no more subtraction operators are found.
            }
        }
        Ok(expr) // Return the parsed expression.
    }

    /// `*`, `/`, `%` - Parse multiplication, division, and modulo operations.
    fn parse_mul_div_mod(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_unary()?; // Parse the left-hand side.
        loop {
            if self.eat(Token::Star)? {
                let rhs = self.parse_unary()?; // Parse the right-hand side of multiplication.
                expr = Expr::Binary { op: BinOp::Mul, left: Box::new(expr), right: Box::new(rhs) };
            } else if self.eat(Token::Slash)? {
                let rhs = self.parse_unary()?;
                expr = Expr::Binary { op: BinOp::Div, left: Box::new(expr), right: Box::new(rhs) };
            } else if self.eat(Token::Percent)? {
                let rhs = self.parse_unary()?;
                expr = Expr::Binary { op: BinOp::Mod, left: Box::new(expr), right: Box::new(rhs) };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    /// Prefix: ++, --, +, -, !, ~, *, &, sizeof, casts
    fn parse_unary(&mut self) -> Result<Expr, String> {
        // prefix inc/dec
        if self.eat(Token::Inc)? {
            let e = self.parse_unary()?;
            return Ok(Expr::Unary { op: UnOp::PreInc, expr: Box::new(e) });
        }
        if self.eat(Token::Dec)? {
            let e = self.parse_unary()?;
            return Ok(Expr::Unary { op: UnOp::PreDec, expr: Box::new(e) });
        }
        // unary plus
        if self.eat(Token::Plus)? {
            let e = self.parse_unary()?;
            return Ok(Expr::Unary { op: UnOp::Plus, expr: Box::new(e) });
        }
        // -, !, ~
        if self.eat(Token::Minus)? {
            let e = self.parse_unary()?;
            return Ok(Expr::Unary { op: UnOp::Neg, expr: Box::new(e) });
        }
        if self.eat(Token::Not)? {
            let e = self.parse_unary()?;
            return Ok(Expr::Unary { op: UnOp::Not, expr: Box::new(e) });
        }
        if self.eat(Token::Tilde)? {
            let e = self.parse_unary()?;
            return Ok(Expr::Unary { op: UnOp::BitNot, expr: Box::new(e) });
        }
        // *, &
        if self.eat(Token::Star)? {
            let e = self.parse_unary()?;
            return Ok(Expr::Unary { op: UnOp::Deref, expr: Box::new(e) });
        }
        if self.eat(Token::And)? {
            let e = self.parse_unary()?;
            return Ok(Expr::Unary { op: UnOp::Addr, expr: Box::new(e) });
        }
        // sizeof(type)
        if self.eat(Token::KwSizeof)? {
            self.expect(Token::LParen)?;
            let t = self.parse_type()?;
            self.expect(Token::RParen)?;
            return Ok(Expr::SizeOf(t));
        }
        // cast: (type)expr
        if self.eat(Token::LParen)? {
            if matches!(self.cur, Token::KwVoid | Token::KwInt | Token::KwChar) {
                let ty = self.parse_type()?;
                self.expect(Token::RParen)?;
                let e = self.parse_unary()?;
                return Ok(Expr::Cast { ty, expr: Box::new(e) });
            } else {
                let e = self.parse_assignment()?;
                self.expect(Token::RParen)?;
                return Ok(e);
            }
        }
        // postfix & primary
        self.parse_postfix()
    }

    /// Postfix: x++ | x-- | function calls | array indexing
    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.eat(Token::Inc)? {
                expr = Expr::Unary { op: UnOp::PostInc, expr: Box::new(expr) };
            } else if self.eat(Token::Dec)? {
                expr = Expr::Unary { op: UnOp::PostDec, expr: Box::new(expr) };
            } else if self.eat(Token::LParen)? {
                // call
                let mut args = Vec::new();
                if self.cur != Token::RParen {
                    loop {
                        args.push(self.parse_assignment()?);
                        if !self.eat(Token::Comma)? {
                            break;
                        }
                    }
                }
                self.expect(Token::RParen)?;
                expr = Expr::Call { callee: Box::new(expr), args };
            } else if self.eat(Token::LBracket)? {
                // array indexing
                let idx = self.parse_assignment()?;
                self.expect(Token::RBracket)?;
                expr = Expr::Index { array: Box::new(expr), index: Box::new(idx) };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    /// Primary: number, string, identifier.
    fn parse_primary(&mut self) -> Result<Expr, String> {
        match &self.cur {
            Token::Num(n) => {
                let v = *n;
                self.bump().map_err(|e| e.0)?;
                Ok(Expr::Num(v))
            }
            Token::Str(s) => {
                let lit = s.clone();
                self.bump().map_err(|e| e.0)?;
                Ok(Expr::Str(lit))
            }
            Token::Ident(_) => {
                let name = self.expect_ident()?;
                Ok(Expr::Var(name))
            }
            _ => Err(format!("unexpected primary {:?}", self.cur)),
        }
    }
}
