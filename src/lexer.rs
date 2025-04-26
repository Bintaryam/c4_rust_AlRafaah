//! Lexer for the C4 compiler subset in Rust.
//! Takes a &str and produces a sequence of Tokens.

use std::iter::Peekable;
use std::str::CharIndices;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    /// int literal (decimal, hex/octal can be added)
    Num(i64),
    /// identifier
    Ident(String),
    /// string literal, with escape processing
    Str(String),
    /// char literal, with escape processing
    Char(char),

    KwVoid,
    KwInt,
    KwChar,
    KwEnum,
    KwIf,
    KwElse,
    KwWhile,
    KwReturn,
    KwSizeof,

    // binary operators and punctuation
    Plus, Minus, Star, Slash, Percent,
    Assign,       // '='
    EqEq, Not, Ne, // '==', '!', '!='
    Lt, Le, Gt, Ge, // '<', '<=', '>', '>='
    And, AndAnd,   // '&', '&&'
    Or, OrOr,      // '|', '||'
    Xor,           // '^'
    Shl, Shr,      // '<<', '>>'
    Inc, Dec,      // '++', '--'

    // bitwise NOT
    Tilde,        // '~'

    // ternary/punctuation
    Question, Colon,
    Semicolon, Comma,
    LParen, RParen,  // '(', ')'
    LBrace, RBrace,  // '{', '}'
    LBracket, RBracket, // '[', ']'

    /// end-of-input marker
    Eof,
}

#[derive(Debug)]
pub struct LexError(pub String);

/// The lexer struct wraps the input string and a peekable index iterator.
pub struct Lexer<'a> {
    input: &'a str,
    iter: Peekable<CharIndices<'a>>,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer instance.
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input,
            iter: input.char_indices().peekable(),
        }
    }

    /// Return the next token or a LexError.
    pub fn next_token(&mut self) -> Result<Token, LexError> {
        self.skip_whitespace_and_comments(); // Skip irrelevant characters.

        let (idx, ch) = match self.iter.peek() {
            Some(&(i, c)) => (i, c),
            None => return Ok(Token::Eof), // End of input.
        };

        self.iter.next(); // Advance the iterator.

        // Handle numeric literals.
        if ch.is_ascii_digit() {
            let start = idx;
            let mut end = idx + ch.len_utf8();
            let mut base = 10;

            // hex & octal support
            if ch == '0' {
                if let Some(&(_, next)) = self.iter.peek() {
                    match next {
                        'x' | 'X' => {
                            base = 16;
                            // consume 'x' or 'X'
                            self.iter.next();
                            end += next.len_utf8();
                            // consume hex digits
                            while let Some(&(_, c)) = self.iter.peek() {
                                if c.is_ascii_hexdigit() {
                                    end += c.len_utf8();
                                    self.iter.next();
                                } else {
                                    break;
                                }
                            }
                        }
                        '0'..='7' => {
                            base = 8;
                            // consume octal digits
                            while let Some(&(_, c)) = self.iter.peek() {
                                if ('0'..='7').contains(&c) {
                                    end += c.len_utf8();
                                    self.iter.next();
                                } else {
                                    break;
                                }
                            }
                        }
                        _ => { /* single '0', leave base=10 */ }
                    }
                }
            } else {
                // decimal: consume more digits
                while let Some(&(_, c)) = self.iter.peek() {
                    if c.is_ascii_digit() {
                        end += c.len_utf8();
                        self.iter.next();
                    } else {
                        break;
                    }
                }
            }

            let slice = &self.input[start..end];
            let val = i64::from_str_radix(slice, base)
                .map_err(|e| LexError(e.to_string()))?;
            return Ok(Token::Num(val));
        }

        // Handle identifiers or keywords.
        if ch.is_ascii_alphabetic() || ch == '_' {
            let mut end = idx;
            while let Some(&(_, c)) = self.iter.peek() {
                if c.is_ascii_alphanumeric() || c == '_' {
                    end += c.len_utf8();
                    self.iter.next();
                } else {
                    break;
                }
            }
            let ident = &self.input[idx..end + ch.len_utf8()];
        
        
        return Ok(match ident {
            "void"   => Token::KwVoid,
            "char"   => Token::KwChar,
            "else"   => Token::KwElse,
            "enum"   => Token::KwEnum,
            "if"     => Token::KwIf,
            "int"    => Token::KwInt,
            "return" => Token::KwReturn,
            "sizeof" => Token::KwSizeof,
            "while"  => Token::KwWhile,
            _        => Token::Ident(ident.to_string()),
            });
        }

        // Handle string literals.
        if ch == '"' {
            let mut s = String::new();
            while let Some(&(_, c)) = self.iter.peek() {
                self.iter.next();
                if c == '"' {
                    break; // End of string.
                }
                if c == '\\' {
                    // Handle escape sequences.
                    if let Some(&(_, esc)) = self.iter.peek() {
                        self.iter.next();
                        match esc {
                            'n' => s.push('\n'), // Newline escape.
                            other => s.push(other), // Other escapes.
                        }
                    }
                } else {
                    s.push(c); // Regular character.
                }
            }
            return Ok(Token::Str(s));
        }

        // Handle character literals.
        if ch == '\'' {
            let c = match self.iter.next().map(|(_, c)| c) {
                Some('\\') => match self.iter.next().map(|(_, c)| c) {
                    Some('n') => '\n', // Newline escape.
                    Some(other) => other, // Other escapes.
                    None => return Err(LexError("Unterminated char literal".into())),
                },
                Some(other) => other, // Regular character.
                None => return Err(LexError("Unterminated char literal".into())),
            };
            // Consume closing single quote.
            if let Some(&(_, '\'')) = self.iter.peek() {
                self.iter.next();
            }
            // fold into Num
            return Ok(Token::Num(c as i64));
        }

        // Handle two-character operators.
        if let Some(&(_, next)) = self.iter.peek() {
            let two = format!("{}{}", ch, next);
            if let Some(tok) = match two.as_str() {
                "==" => Some(Token::EqEq),
                "!=" => Some(Token::Ne),
                "<=" => Some(Token::Le),
                ">=" => Some(Token::Ge),
                "&&" => Some(Token::AndAnd),
                "||" => Some(Token::OrOr),
                "<<" => Some(Token::Shl),
                ">>" => Some(Token::Shr),
                "++" => Some(Token::Inc),
                "--" => Some(Token::Dec),
                _ => None,
            } {
                self.iter.next(); // Consume the second character.
                return Ok(tok);
            }
        }

        // Handle single-character tokens.
        let tok = match ch {
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Star,
            '/' => Token::Slash,
            '%' => Token::Percent,
            '=' => Token::Assign,
            '!' => Token::Not,
            '<' => Token::Lt,
            '>' => Token::Gt,
            '&' => Token::And,
            '|' => Token::Or,
            '^' => Token::Xor,
            '~' => Token::Tilde,   // support '~'
            '?' => Token::Question,
            ':' => Token::Colon,
            ';' => Token::Semicolon,
            ',' => Token::Comma,
            '(' => Token::LParen,
            ')' => Token::RParen,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            _ => return Err(LexError(format!("Unexpected character '{}'", ch))),
        };
        Ok(tok)
    }

    /// Skip whitespace, comments, and preprocessor lines in the input.
    fn skip_whitespace_and_comments(&mut self) {
        while let Some(&(_, c)) = self.iter.peek() {
            if c.is_whitespace() {
                self.iter.next(); // Skip whitespace.
            } else if c == '/' {
                // Check for comments.
                let mut clone = self.iter.clone();
                clone.next();
                if let Some(&(_, '/')) = clone.peek() {
                    // Consume "//" and the rest of the line.
                    self.iter.next();
                    self.iter.next();
                    while let Some(&(_, c2)) = self.iter.peek() {
                        self.iter.next();
                        if c2 == '\n' {
                            break; // End of comment.
                        }
                    }
                } else {
                    break; // Not a comment.
                }
            } else if c == '#' {
                // Consume preprocessor line.
                self.iter.next();
                while let Some(&(_, c2)) = self.iter.peek() {
                    self.iter.next();
                    if c2 == '\n' {
                        break;
                    }
                }
            } else {
                break; // Stop skipping.
            }
        }
    }
}
