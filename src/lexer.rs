//! Lexical analyzer for the Slvr language
//!
//! Tokenizes source code into a stream of tokens.

use crate::error::{SlvrError, SlvrResult};
use serde::{Deserialize, Serialize};

/// Token types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenType {
    // Literals
    Integer(i128),
    Decimal(f64),
    String(String),
    Identifier(String),

    // Keywords
    Module,
    Defun,
    Defschema,
    Deftable,
    Defconst,
    If,
    Let,
    Read,
    Write,
    Update,
    Delete,
    True,
    False,
    Null,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    Equal,
    EqualEqual,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    Not,
    Concat,

    // Delimiters
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Colon,
    Semicolon,
    Comma,
    Dot,
    Arrow,

    // Special
    Eof,
}

/// A token with location information
#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, line: usize, column: usize) -> Self {
        Self {
            token_type,
            line,
            column,
        }
    }
}

/// Lexical analyzer
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    /// Create a new lexer from source code
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    /// Get the next token
    pub fn next_token(&mut self) -> SlvrResult<Token> {
        self.skip_whitespace_and_comments();

        if self.position >= self.input.len() {
            return Ok(Token::new(TokenType::Eof, self.line, self.column));
        }

        let ch = self.current_char();
        let line = self.line;
        let column = self.column;

        let token_type = match ch {
            '(' => {
                self.advance();
                TokenType::LeftParen
            }
            ')' => {
                self.advance();
                TokenType::RightParen
            }
            '[' => {
                self.advance();
                TokenType::LeftBracket
            }
            ']' => {
                self.advance();
                TokenType::RightBracket
            }
            '{' => {
                self.advance();
                TokenType::LeftBrace
            }
            '}' => {
                self.advance();
                TokenType::RightBrace
            }
            ':' => {
                self.advance();
                TokenType::Colon
            }
            ';' => {
                self.advance();
                TokenType::Semicolon
            }
            ',' => {
                self.advance();
                TokenType::Comma
            }
            '.' => {
                self.advance();
                TokenType::Dot
            }
            '+' => {
                self.advance();
                if self.current_char() == '+' {
                    self.advance();
                    TokenType::Concat
                } else {
                    TokenType::Plus
                }
            }
            '-' => {
                self.advance();
                if self.current_char() == '>' {
                    self.advance();
                    TokenType::Arrow
                } else {
                    TokenType::Minus
                }
            }
            '*' => {
                self.advance();
                TokenType::Star
            }
            '/' => {
                self.advance();
                TokenType::Slash
            }
            '%' => {
                self.advance();
                TokenType::Percent
            }
            '^' => {
                self.advance();
                TokenType::Caret
            }
            '=' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                }
            }
            '!' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    TokenType::NotEqual
                } else {
                    TokenType::Not
                }
            }
            '<' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                }
            }
            '>' => {
                self.advance();
                if self.current_char() == '=' {
                    self.advance();
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                }
            }
            '&' => {
                self.advance();
                if self.current_char() == '&' {
                    self.advance();
                    TokenType::And
                } else {
                    return Err(SlvrError::lexer(line, column, "unexpected character '&'"));
                }
            }
            '|' => {
                self.advance();
                if self.current_char() == '|' {
                    self.advance();
                    TokenType::Or
                } else {
                    return Err(SlvrError::lexer(line, column, "unexpected character '|'"));
                }
            }
            '"' => self.read_string()?,
            _ if ch.is_ascii_digit() => self.read_number()?,
            _ if ch.is_alphabetic() || ch == '_' => self.read_identifier(),
            _ => return Err(SlvrError::lexer(line, column, format!("unexpected character '{}'", ch))),
        };

        Ok(Token::new(token_type, line, column))
    }

    /// Tokenize the entire input
    pub fn tokenize(&mut self) -> SlvrResult<Vec<Token>> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            let is_eof = matches!(token.token_type, TokenType::Eof);
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }

    fn current_char(&self) -> char {
        if self.position < self.input.len() {
            self.input[self.position]
        } else {
            '\0'
        }
    }

    fn peek_char(&self, offset: usize) -> char {
        if self.position + offset < self.input.len() {
            self.input[self.position + offset]
        } else {
            '\0'
        }
    }

    fn advance(&mut self) {
        if self.position < self.input.len() {
            if self.input[self.position] == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.position += 1;
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        while self.position < self.input.len() {
            match self.current_char() {
                ' ' | '\t' | '\n' | '\r' => self.advance(),
                ';' => {
                    // Skip line comment
                    while self.position < self.input.len() && self.current_char() != '\n' {
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    fn read_string(&mut self) -> SlvrResult<TokenType> {
        let line = self.line;
        let column = self.column;
        self.advance(); // Skip opening quote

        let mut result = String::new();
        while self.position < self.input.len() && self.current_char() != '"' {
            if self.current_char() == '\\' {
                self.advance();
                match self.current_char() {
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    'r' => result.push('\r'),
                    '\\' => result.push('\\'),
                    '"' => result.push('"'),
                    _ => result.push(self.current_char()),
                }
            } else {
                result.push(self.current_char());
            }
            self.advance();
        }

        if self.position >= self.input.len() {
            return Err(SlvrError::lexer(line, column, "unterminated string"));
        }

        self.advance(); // Skip closing quote
        Ok(TokenType::String(result))
    }

    fn read_number(&mut self) -> SlvrResult<TokenType> {
        let mut result = String::new();
        let mut is_decimal = false;

        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch.is_ascii_digit() {
                result.push(ch);
                self.advance();
            } else if ch == '.' && !is_decimal {
                is_decimal = true;
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if is_decimal {
            let value = result
                .parse::<f64>()
                .map_err(|_| SlvrError::lexer(self.line, self.column, "invalid decimal"))?;
            Ok(TokenType::Decimal(value))
        } else {
            let value = result
                .parse::<i128>()
                .map_err(|_| SlvrError::lexer(self.line, self.column, "invalid integer"))?;
            Ok(TokenType::Integer(value))
        }
    }

    fn read_identifier(&mut self) -> TokenType {
        let mut result = String::new();
        while self.position < self.input.len() {
            let ch = self.current_char();
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        match result.as_str() {
            "module" => TokenType::Module,
            "defun" => TokenType::Defun,
            "defschema" => TokenType::Defschema,
            "deftable" => TokenType::Deftable,
            "defconst" => TokenType::Defconst,
            "if" => TokenType::If,
            "let" => TokenType::Let,
            "read" => TokenType::Read,
            "write" => TokenType::Write,
            "update" => TokenType::Update,
            "delete" => TokenType::Delete,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "null" => TokenType::Null,
            _ => TokenType::Identifier(result),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokens() {
        let mut lexer = Lexer::new("()[]{}");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 7); // 6 tokens + EOF
    }

    #[test]
    fn test_numbers() {
        let mut lexer = Lexer::new("42 3.14");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0].token_type, TokenType::Integer(42)));
        assert!(matches!(tokens[1].token_type, TokenType::Decimal(_)));
    }

    #[test]
    fn test_strings() {
        let mut lexer = Lexer::new("\"hello world\"");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0].token_type, TokenType::String(_)));
    }

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("defun if let true false");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0].token_type, TokenType::Defun));
        assert!(matches!(tokens[1].token_type, TokenType::If));
    }
}
