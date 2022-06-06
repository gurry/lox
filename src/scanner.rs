use std::fmt::Display;

use thiserror::Error;
use anyhow::{Result, bail, Context};

#[derive(Error, Debug)]
#[error("[{line}]: {message}")]
pub struct ScanError {
	pub line: usize,
    pub message: String
}

#[derive(Debug)]
pub struct Scanner {
    source: String,
    start: usize,
    current: usize,
    line: usize
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self { source, start: 0, current: 0, line: 1 }
    }

    pub fn scan_next(&mut self) -> Result<Token> {
        self.skip_whitespace();

        if self.is_at_end() {
            return Ok(Token { lexeme: "", line: self.line, token_type: TokenType::Eof });
        }

        let token_type = self.scan_token()?;

        let lexeme = self.current_lexeme();

        Ok(Token { token_type, lexeme, line: self.line })
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                '\n' => {
                    self.line += 1;
                    self.advance();
                },
                ' ' | '\r' | '\t' => { self.advance(); },
                '/' => { 
                    if self.peek_next() == '/' { // A commit starts with two slaces.
                        // A comment goes until the end of the line.
                        while self.peek() != '\n' && !self.is_at_end() {
                            self.advance();
                        }
                    }
                    else {
                        break
                    }
                },
                _ => break
            }
        }
    } 

    fn scan_token(&mut self) -> Result<TokenType> {
        self.start = self.current;
        let c = self.advance();

        let token_type = match c {
            '(' => TokenType::LeftParen,
            ')' => TokenType::RightParen,
            '{' => TokenType::LeftBrace,
            '}' => TokenType::RightBrace,
            ',' => TokenType::Comma,
            '.' => TokenType::Dot,
            '-' => TokenType::Minus,
            '+' => TokenType::Plus,
            ';' => TokenType::Semicolon,
            '*' => TokenType::Star,
            '!' => if self.char_matches('=') { TokenType::BangEqual } else { TokenType::Bang },
            '=' => if self.char_matches('=') { TokenType::EqualEqual } else { TokenType::Equal },
            '<' => if self.char_matches('=') { TokenType::LessEqual } else { TokenType::Less },
            '>' => if self.char_matches('=') { TokenType::GreaterEqual } else { TokenType::Greater },
            '/' => TokenType::Slash,
            '0'..='9' => self.number()?,
            '"' => self.string()?,
            c => {
                if self.is_alpha(c) {
                    self.identifier()
                }
                else {
                    bail!(ScanError { line: self.line, message: "Unexpected character.".to_string() })
                }
            }
        };

        Ok(token_type)
    }

    fn string(&mut self) -> Result<TokenType> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            bail!(ScanError { line: self.line, message: "Unterminated string.".to_string() });
        }

        // The closing ".
        self.advance();

            
        let lexeme = self.current_lexeme();
        let lexeme_without_quotes = &lexeme[1..(lexeme.len() - 1)]; // Trim the surrounding quotes.
        Ok(TokenType::String(lexeme_without_quotes.to_string()))
    }

    fn number(&mut self) -> Result<TokenType> {
        while self.is_digit(self.peek()) {
            self.advance();
        }

        // Look for a fractional part.
        if self.peek() == '.' && self.is_digit(self.peek_next()) {
            // Consume the "."
            self.advance();
    
            while self.is_digit(self.peek()) {
                self.advance();
            }
        }
    
        let substr = self.current_lexeme();
        let value =  substr.parse::<f64>()
            .context(format!("Failed to parse '{}' as number", substr))?;
        Ok(TokenType::Number(value))
    }

    fn identifier(&mut self) -> TokenType {
        while self.is_alphanumeric(self.peek()) {
             self.advance();
        }

        match self.current_lexeme() {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "fun" => TokenType::Fun,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            _ => TokenType::Identifier,
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn is_digit(&self, c: char) -> bool {
        c >= '0' && c <= '9'
    }
    fn is_alpha(&self, c: char) -> bool {
        (c >= 'a' && c <= 'z') ||
        (c >= 'A' && c <= 'Z') ||
        c == '_'
    }
    
    fn is_alphanumeric(&self, c: char) -> bool {
        self.is_alpha(c) || self.is_digit(c)
    }

    fn char_matches(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        } 

        if self.source.chars().nth(self.current) != Some(expected) {
            return false;
        }

        self.current += 1;
        true
    }

    fn advance(&mut self) -> char {
        let c = self.current_char();
        self.current += 1;
        c
    }

    fn peek(&self) -> char {
        if self.is_at_end() { '\0' } else { self.current_char() }
    }

    fn peek_next(&self) -> char {
        match self.char_at(self.current + 1) {
            Some(c) => c,
            None => '\0'
        }
    }

    fn current_lexeme(&self) -> &str {
        &self.source[self.start..self.current]
    }

    fn current_char(&self) -> char {
        self.char_at(self.current).expect("Ran past end of source")
    }

    fn char_at(&self, index: usize) -> Option<char> {
        if index >= self.source.len() {
            None
        } else {
            Some(self.source.as_bytes()[index] as char)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token<'a> {
    pub token_type: TokenType,
    pub lexeme: &'a str,
    pub line: usize
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at line {}", self.lexeme, self.line)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    LeftParen, RightParen, LeftBrace, RightBrace, Comma,
    Dot, Minus, Plus, Semicolon, Slash, Star,

    Bang, BangEqual, Equal, EqualEqual, Greater, GreaterEqual,
    Less, LessEqual,

    Identifier, String(String), Number(f64),

    And, Class, Else, False, Fun, For, If, Nil, Or, Print,
    Return, Super, This, True, Var, While,

    Eof
}