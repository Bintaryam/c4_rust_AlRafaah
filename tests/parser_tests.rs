// tests/parser_tests.rs

// Replace `your_crate` with the actual name from Cargo.toml, e.g.
// `use c4_rust_alrafaah::{ast::*, parser::Parser};`
use your_crate::{ast::*, parser::Parser};

/// Helper: parse a full program into an AST or panic.
fn parse_to_ast(src: &str) -> Program {
    Parser::new(src)
        .and_then(|mut p| p.parse_program())
        .expect("parsing failed")
}

#[test]
fn parse_global_and_enum_decls() {
    let src = r#"
        int a, b;
        enum { X = 1, Y, Z = 5 };
        char *p;
    "#;
    let Program { items } = parse_to_ast(src);
    assert!(matches!(items[0], Item::Global(_)));
    assert!(matches!(items[2], Item::Enum(_)));
    assert!(matches!(items[3], Item::Global(_)));

    // Check enum variants
    if let Item::Enum(EnumDecl { variants }) = &items[2] {
        assert_eq!(variants.len(), 3);
        assert_eq!(variants[0], ("X".into(), Some(1)));
        assert_eq!(variants[1], ("Y".into(), None));
        assert_eq!(variants[2], ("Z".into(), Some(5)));
    } else {
        panic!("expected enum declaration");
    }
}

#[test]
fn parse_function_and_statements() {
    let src = r#"
        void foo(int x, char y) {
            ;                // empty stmt
            {
                int z;
                z = x + y;
            }
            if (z > 0) z--;
            else ++z;
            while (x != 0) { x = x - 1; }
            return;
        }
    "#;
    let Program { items } = parse_to_ast(src);
    let func = match &items[0] {
        Item::Function(f) => f,
        _ => panic!("expected function"),
    };

    // signature
    assert_eq!(func.ret, Type::Void);
    assert_eq!(func.params, vec![("x".into(), Type::Int), ("y".into(), Type::Char)]);
    // body stmts count
    assert_eq!(func.body.stmts.len(), 6);

    // 0: Empty
    assert!(matches!(func.body.stmts[0], Stmt::Empty));

    // 1: Nested block with local and assignment
    if let Stmt::Block(Block { stmts }) = &func.body.stmts[1] {
        assert!(matches!(stmts[0], Stmt::Expr(_)));
    } else {
        panic!("expected nested block");
    }

    // 2: If / else
    if let Stmt::If { cond, then_branch, else_branch } = &func.body.stmts[2] {
        assert!(matches!(**cond, Expr::Binary { op: BinOp::Gt, .. }));
        assert!(matches!(*then_branch.clone(), Stmt::Expr(_)));
        assert!(matches!(*else_branch.clone().unwrap(), Stmt::Expr(_)));
    } else {
        panic!("expected if stmt");
    }

    // 3: While
    assert!(matches!(func.body.stmts[3], Stmt::While { .. }));

    // 4: Return without expr
    assert!(matches!(func.body.stmts[4], Stmt::Return(None)));
}

#[test]
fn parse_unary_and_postfix_ops() {
    let src = r#"
        int u() {
          return +x - !y * ~z & *p | &q;
        }
    "#;
    let Program { items } = parse_to_ast(src);
    let body = match &items[0] { Item::Function(f) => &f.body, _ => panic!() };
    // Expr: Binary chain ((+x) - (!y)) * (~z) & (*p) | (&q)
    if let Stmt::Return(Some(expr)) = &body.stmts[0] {
        // Top-level op is BitOr
        assert!(matches!(expr, Expr::Binary { op: BinOp::BitOr, .. }));
    } else {
        panic!("expected return expr");
    }

    // Postfix ++/--:
    let src2 = "int p() { return a++ + --b; }";
    let Program { items } = parse_to_ast(src2);
    let expr = if let Stmt::Return(Some(e)) = &match &items[0] { Item::Function(f) => &f.body.stmts[0], _ => panic!() } { e } else { panic!() };
    // Check PostInc and PreDec appear
    if let Expr::Binary { left, right, .. } = expr {
        assert!(matches!(**left, Expr::Unary { op: UnOp::PostInc, .. }));
        assert!(matches!(**right, Expr::Unary { op: UnOp::PreDec, .. }));
    } else {
        panic!("expected binary");
    }
}

#[test]
fn parse_sizeof_and_cast() {
    let src = "int c() { return sizeof(int) + sizeof(char*) + (char*)p; }";
    let Program { items } = parse_to_ast(src);
    let expr = if let Stmt::Return(Some(e)) = &match &items[0] { Item::Function(f) => &f.body.stmts[0], _ => panic!() } { e } else { panic!() };
    // should be a Binary chain; check one SizeOf and one Cast
    let found_sizeof = format!("{:?}", expr).contains("SizeOf");
    let found_cast = format!("{:?}", expr).contains("Cast");
    assert!(found_sizeof && found_cast);
}

#[test]
fn parse_shifts_and_bitwise() {
    let src = "int s() { return a << 2 >> 1 & b | c ^ d; }";
    let Program { items } = parse_to_ast(src);
    let expr = if let Stmt::Return(Some(e)) = &match &items[0] { Item::Function(f) => &f.body.stmts[0], _ => panic!() } { e } else { panic!() };
    let repr = format!("{:?}", expr);
    assert!(repr.contains("Shl") && repr.contains("Shr") && repr.contains("BitAnd"));
    assert!(repr.contains("BitOr") && repr.contains("Xor"));
}

#[test]
fn parse_ternary_and_logical() {
    let src = "int t() { return a ? b : c && d || e; }";
    let Program { items } = parse_to_ast(src);
    let stmt = &match &items[0] { Item::Function(f) => &f.body.stmts[0], _ => panic!() };
    if let Stmt::Return(Some(Expr::Binary { op: BinOp::LogOr, left, right })) = stmt {
        assert!(matches!(**right, Expr::Var(ref s) if s == "e"));
        if let Expr::Binary { op: BinOp::LogAnd, left: la, right: lb } = &**left {
            assert!(matches!(**lb, Expr::Var(ref s) if s == "d"));
            // la must be Conditional
            assert!(matches!(**la, Expr::Conditional { .. }));
        } else {
            panic!("expected logical AND inside OR");
        }
    } else {
        panic!("expected logical OR at top");
    }
}

#[test]
fn parse_indexing_and_calls_and_strings() {
    let src = r#"
      int w() {
        return foo("hi\n", arr[0], x * y);
      }
    "#;
    let Program { items } = parse_to_ast(src);
    let stmt = &match &items[0] { Item::Function(f) => &f.body.stmts[0], _ => panic!() };
    if let Stmt::Return(Some(Expr::Call { callee, args })) = stmt {
        assert!(matches!(*callee.clone(), Expr::Var(ref s) if s == "foo"));
        // args: Str, Index, Binary
        assert!(matches!(args[0], Expr::Str(_)));
        assert!(matches!(args[1], Expr::Index { .. }));
        assert!(matches!(args[2], Expr::Binary { op: BinOp::Mul, .. }));
    } else {
        panic!("expected call in return");
    }
}
