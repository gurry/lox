use std::fmt::Display;

use anyhow::{Result, bail};
use thiserror::Error;
use crate::{scanner::{Scanner, Token, ScanError, TokenType}, chunk::Chunk, instruction::OpCode};

pub struct Compiler{
    scanner: Scanner,
    current_token: Option<Token>,
    prev_token: Option<Token>,
    errors: Vec<CompileError>,
    panic_mode: bool
}

impl Compiler {
    pub fn new(source: String) -> Self {
        Self { scanner: Scanner::new(source), current_token: None, prev_token: None, errors: Vec::new(), panic_mode: false }
    }

    pub fn compile(&mut self) -> Result<Chunk> {
        let mut chunk = Chunk::new();

        self.advance();

        self.expression();

        self.consume(&TokenType::Eof, "Expected EOF");

        if !self.errors.is_empty() {
            bail!(CompileErrorCollection { errors: self.errors.clone() })
        }

        let line = match &self.current_token {
            Some(t) => t.line,
            None => 0,
        };

        chunk.write(OpCode::Return, line as i32);

        Ok(chunk)
    } 

    fn expression(&mut self) {
    }

    fn advance(&mut self) {
        self.prev_token = self.current_token.take();

        self.current_token = loop {
            match self.scanner.scan_next()
            {
                Ok(token) => {
                    break Some(token)
                },
                Err(e) => {
                    let scan_err = e.downcast_ref::<ScanError>().unwrap();
                    self.push_scan_error(scan_err);
                }
            }
        };
    }

    fn consume(&mut self, token_type: &TokenType, message: &str) {
        if let Some(curr_token) = &self.current_token {
            if curr_token.token_type == *token_type {
                return self.advance();
            }

            self.push_current_parse_error(message)
        } else {
            self.push_current_parse_error(format!("Expected {:?} but no current token", token_type))
        }
        
    }

    fn push_current_parse_error<M: Into<String>>(&mut self, msg: M) {
        let current_token = self.current_token.as_ref().expect("No current token by trying to push parse error");
        self.push_parse_error(msg, current_token.clone())
    }

    fn push_parse_error<M: Into<String>>(&mut self, msg: M, token: Token) {
        let lexeme = self.scanner.get_lexeme_str(&token.lexeme)
            .expect("Lexeme outside of source boundary");
        self.push_error(CompileError::new_parse_error(msg, lexeme.to_string(), token.line))
    }

    fn push_scan_error(&mut self, scan_err: &ScanError) {
        self.push_error(CompileError::Scan(scan_err.clone()))
    }

    fn push_error(&mut self, error: CompileError) {
        if !self.panic_mode {
            self.errors.push(error);
            self.panic_mode = true;
        }
    }
}

#[derive(Error, Clone, Debug)]
pub struct CompileErrorCollection {
    pub errors: Vec<CompileError>
}

impl Display for CompileErrorCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for e in &self.errors {
            writeln!(f, "{}", e)?;
        }

        Ok(())
    }
}

#[derive(Error, Clone, Debug)]
pub enum CompileError {
    #[error("[line {line}] Compile error: '{lexeme}' - {msg}")]
    Parse {
        msg: String,
        lexeme: String,
        line: usize 
    },
    #[error("{0}")]
    Scan(ScanError)
}

impl CompileError {
    pub fn new_parse_error<M: Into<String>>(msg: M, lexeme: String, line:usize) -> Self { 
        Self::Parse { msg: msg.into(), lexeme, line }
    }
}   