use crate::token::{TokenType, Token};
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
        loop {
            if let Some(token) = self.scan_token()? {
                break Ok(token)
            }
        }
    }

    fn scan_token(&mut self) -> Result<Option<Token>> {
        if self.is_at_end() {
            return Ok(Some(Token { lexeme: "".to_string(), line: self.line, token_type: TokenType::Eof }));
        }

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
            '/' => { 
                if self.char_matches('/') {
                    // A comment goes until the end of the line.
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }

                    return Ok(None)
                } else {
                    TokenType::Slash
                }
            },
            '0'..='9' => self.number()?,
            '"' => self.string()?,
            ' ' | '\r' | '\t' => return Ok(None),
            '\n' => {
                self.line += 1;
                return Ok(None)
            },
            c => {
                if self.is_alpha(c) {
                    self.identifier()
                }
                else {
                    bail!(ScanError { line: self.line, message: "Unexpected character.".to_string() })
                }
            }
        };

        let lexeme: String = self.source.chars().into_iter().skip(self.start).take(self.current - self.start).collect();

        Ok(Some(Token { token_type, lexeme, line: self.line }))
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

        // Trim the surrounding quotes.
            
        let value: String = self.source.chars().into_iter().skip(self.start + 1).take(self.current - self.start - 2).collect();
        Ok(TokenType::String(value))
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
    
        let substr: String = self.source.chars().into_iter().skip(self.start).take(self.current - self.start).collect();
        let value =  substr.parse::<f64>()
            .context(format!("Failed to parse '{}' as number", substr))?;
        Ok(TokenType::Number(value))
    }

    fn identifier(&mut self) -> TokenType {
        while self.is_alphanumeric(self.peek()) {
             self.advance();
        }

        let substr: String = self.source.chars().into_iter().skip(self.start).take(self.current - self.start).collect();

        match substr.as_str() {
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
        let c = self.source.chars().nth(self.current);
        self.current += 1;
        c.expect("Ran out of chars to advance to")
    }

    fn peek(&self) -> char {
        if self.is_at_end() { '\0' } else { self.source.chars().nth(self.current).expect("Char can't be None") }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        self.source.chars().nth(self.current + 1).expect("Char can't be None")
    } 
}