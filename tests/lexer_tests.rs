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
    // Test lexing of character literals, including escape sequences.
    let mut lx = Lexer::new(r" 'a' '\n' ");
    assert_eq!(lx.next_token().unwrap(), Token::Char('a'));
    assert_eq!(lx.next_token().unwrap(), Token::Char('\n'));
    assert_eq!(lx.next_token().unwrap(), Token::Eof);
}

#[test]
fn skip_comments_and_whitespace() {
    // Test skipping of comments and whitespace.
    let src = "  // this is a comment\n  42 // another\n+\n";
    let mut lx = Lexer::new(src);
    assert_eq!(lx.next_token().unwrap(), Token::Num(42)); // Number after comment.
    assert_eq!(lx.next_token().unwrap(), Token::Plus); // Operator after whitespace.
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
