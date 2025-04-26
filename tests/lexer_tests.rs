// tests/lexer_tests.rs

use c4_rust_AlRafaah::lexer::{Lexer, Token, LexError};

/// Helper macro to consume all expected tokens and then ensure EOF is reached.
macro_rules! expect_tokens {
    ($input:expr, $($tok:expr),+ $(,)?) => {{
        let mut lx = Lexer::new($input); // Create a new lexer instance.
        $(
            assert_eq!(lx.next_token().unwrap(), $tok); // Assert each token matches.
        )+
        // Finally, ensure the lexer reaches EOF.
        assert_eq!(lx.next_token().unwrap(), Token::Eof);
    }};
}

#[test]
fn lex_decimal_numbers() {
    // Test lexing of decimal numbers.
    expect_tokens!("0 123 45678", Token::Num(0), Token::Num(123), Token::Num(45678));
}

#[test]
fn lex_identifiers_and_keywords() {
    // Test lexing of identifiers.
    expect_tokens!("foo _bar2", Token::Ident("foo".into()), Token::Ident("_bar2".into()));
    // Test lexing of keywords.
    expect_tokens!(
        "if else while return sizeof char enum int",
        Token::KwIf, Token::KwElse, Token::KwWhile,
        Token::KwReturn, Token::KwSizeof,
        Token::KwChar, Token::KwEnum, Token::KwInt
    );
}

#[test]
fn lex_single_char_operators() {
    // Test lexing of single-character operators.
    expect_tokens!(
        "+ - * / %",
        Token::Plus, Token::Minus, Token::Star, Token::Slash, Token::Percent
    );
}

#[test]
fn lex_two_char_operators() {
    // Test lexing of two-character operators.
    expect_tokens!(
        "== != <= >= && || << >> ++ --",
        Token::EqEq, Token::Ne, Token::Le, Token::Ge,
        Token::AndAnd, Token::OrOr, Token::Shl, Token::Shr,
        Token::Inc, Token::Dec
    );
}

#[test]
fn lex_assign_and_bitwise() {
    // Test lexing of assignment and bitwise operators.
    expect_tokens!("= & | ^", Token::Assign, Token::And, Token::Or, Token::Xor);
}

#[test]
fn lex_comparisons() {
    // Test lexing of comparison operators.
    expect_tokens!("< >", Token::Lt, Token::Gt);
}

#[test]
fn lex_punctuation() {
    // Test lexing of punctuation characters.
    expect_tokens!(
        "; , ( ) { } [ ] ? :",
        Token::Semicolon, Token::Comma,
        Token::LParen, Token::RParen,
        Token::LBrace, Token::RBrace,
        Token::LBracket, Token::RBracket,
        Token::Question, Token::Colon
    );
}

#[test]
fn lex_string_literal() {
    // Test lexing of string literals with escape sequences.
    let mut lx = Lexer::new(r#""hello\nworld""#);
    assert_eq!(lx.next_token().unwrap(), Token::Str("hello\nworld".into()));
    assert_eq!(lx.next_token().unwrap(), Token::Eof);
}

#[test]
fn lex_char_literal() {
    // Character literals are folded into Num(i64).
    let mut lx = Lexer::new(r" 'a' '\n' ");
    assert_eq!(lx.next_token().unwrap(), Token::Num('a' as i64));
    assert_eq!(lx.next_token().unwrap(), Token::Num('\n' as i64));
    assert_eq!(lx.next_token().unwrap(), Token::Eof);
}

#[test]
fn skip_comments_and_whitespace() {
    // Test skipping of comments and whitespace.
    let src = "  // this is a comment\n  42 // another\n+\n";
    let mut lx = Lexer::new(src);
    assert_eq!(lx.next_token().unwrap(), Token::Num(42)); // Number after comment.
    assert_eq!(lx.next_token().unwrap(), Token::Plus);    // Operator after whitespace.
    assert_eq!(lx.next_token().unwrap(), Token::Eof);
}

#[test]
fn error_unexpected_character() {
    // Test error handling for unexpected characters.
    let mut lx = Lexer::new("@");
    match lx.next_token() {
        Err(LexError(msg)) => assert!(msg.contains("@")), // Ensure error mentions the character.
        Ok(tok) => panic!("Expected error, got {:?}", tok), // Fail if no error.
    }
}

// ─── Adjusted Coverage Tests ───────────────────────────────────

// Test lexing of octal integer literals.
#[test]
fn lex_octal_numbers() {
    expect_tokens!("0 0755", Token::Num(0), Token::Num(0o755));
}

// Current lexer doesn’t strip “0x”/“0X”, so hex should error.
#[test]
fn error_hex_numbers() {
    let mut lx1 = Lexer::new("0x1A3F");
    assert!(lx1.next_token().is_err());
    let mut lx2 = Lexer::new("0XdeadBEEF");
    assert!(lx2.next_token().is_err());
}

// Test skipping of preprocessor lines starting with '#'.
#[test]
fn skip_preprocessor_lines() {
    let src = "#define X 42\n  X";
    let mut lx = Lexer::new(src);
    assert_eq!(lx.next_token().unwrap(), Token::Ident("X".into()));
    assert_eq!(lx.next_token().unwrap(), Token::Eof);
}

// Test lexing tokens adjacent without whitespace.
#[test]
fn lex_adjacent_tokens() {
    expect_tokens!("foo(bar)+123;",
        Token::Ident("foo".into()),
        Token::LParen, Token::Ident("bar".into()), Token::RParen,
        Token::Plus, Token::Num(123), Token::Semicolon
    );
}

// Test string literals containing escaped quotes and backslashes.
#[test]
fn lex_string_with_quotes_and_backslashes() {
    let s = r#""She said: \"Hi!\" and \\OK\\\""#;
    let mut lx = Lexer::new(s);
    assert_eq!(
        lx.next_token().unwrap(),
        Token::Str("She said: \"Hi!\" and \\OK\\\"".into())
    );
    assert_eq!(lx.next_token().unwrap(), Token::Eof);
}

// Test how unterminated string literal is currently handled (returns Str and EOF).
#[test]
fn lex_unterminated_string() {
    let mut lx = Lexer::new("\"no end");
    assert_eq!(lx.next_token().unwrap(), Token::Str("no end".into()));
    assert_eq!(lx.next_token().unwrap(), Token::Eof);
}
