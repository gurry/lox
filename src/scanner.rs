use crate::token::{TokenType, Token};

pub struct ScanError {
	pub line: usize,
    pub message: String
}

type Result<T> = std::result::Result<T, ScanError>;
#[derive(Debug)]
pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self { source, tokens: Vec::new(), start: 0, current: 0, line: 1 }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }

        self.tokens.push(Token { token_type: TokenType::Eof, lexeme: "".to_string(), line: self.line });
        Ok(self.tokens.clone())
    }

    fn scan_token(&mut self) -> Result<()> {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '!' => {
                let token_type = if self.char_matches('=') { TokenType::BangEqual } else { TokenType::Bang };
                self.add_token(token_type);
            },
            '=' => {
                let token_type = if self.char_matches('=') { TokenType::EqualEqual } else { TokenType::Equal };
                self.add_token(token_type);
            },
            '<' => { 
                let token_type = if self.char_matches('=') { TokenType::LessEqual } else { TokenType::Less };
                self.add_token(token_type);
            },
            '>' => {
                let token_type = if self.char_matches('=') { TokenType::GreaterEqual } else { TokenType::Greater };
                self.add_token(token_type);
            },
            '/' => { 
                if self.char_matches('/') {
                    // A comment goes until the end of the line.
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash);
                }
            },
            '0'..='9' => self.number(),
            '"' => self.string()?,
            ' ' | '\r' | '\t' => { /* Ignore whitespace */},
            '\n' => self.line += 1,
            c => {
                if self.is_alpha(c) {
                    self.identifier();
                }
                else {
                    return Err(ScanError { line: self.line, message: "Unexpected character.".to_string() });
                }
            }
        }

        Ok(())
    }

    fn string(&mut self) -> Result<()> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(ScanError { line: self.line, message: "Unterminated string.".to_string() });
        }

        // The closing ".
        self.advance();

        // Trim the surrounding quotes.
            
        let value: String = self.source.chars().into_iter().skip(self.start + 1).take(self.current - self.start - 2).collect();
        self.add_token(TokenType::String(value));

        Ok(())
    }

    fn number(&mut self) {
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
        let value =  substr.parse::<f64>().expect("Cannot fail to parse number");
        self.add_token(TokenType::Number(value));
    }

    fn identifier(&mut self) {
        while self.is_alphanumeric(self.peek()) {
             self.advance();
        }

        let substr: String = self.source.chars().into_iter().skip(self.start).take(self.current - self.start).collect();

        let token = match substr.as_str() {
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
        };
    
        self.add_token(token);
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

    fn add_token(&mut self, token_type: TokenType) {
        let lexeme: String = self.source.chars().into_iter().skip(self.start).take(self.current - self.start).collect();
        self.tokens.push(Token { token_type, lexeme, line: self.line });
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