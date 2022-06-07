use core::panic;
use std::fmt::Display;

use anyhow::{Result, bail, Context};
use thiserror::Error;
use crate::{scanner::{Scanner, Token, ScanError, TokenType}, chunk::Chunk, instruction::{OpCode, InstructionWriter}};

pub struct Compiler{
    scanner: Scanner,
    writer: InstructionWriter,
    current_token: Option<Token>,
    prev_token: Option<Token>,
    errors: Vec<CompileError>,
    panic_mode: bool
}

impl Compiler {
    pub fn new(source: String) -> Self {
        Self { scanner: Scanner::new(source), writer: InstructionWriter::with_new_chunk(), current_token: None, prev_token: None, errors: Vec::new(), panic_mode: false }
    }

    pub fn compile(mut self) -> Result<Chunk> {
        self.advance();

        self.expression()?;

        self.consume(&TokenType::Eof, "Expected EOF");

        if !self.errors.is_empty() {
            bail!(CompileErrorCollection { errors: self.errors.clone() })
        }

        let line = match &self.current_token {
            Some(t) => t.line,
            None => 0,
        };

        self.writer.write_op_code(OpCode::Return, line as i32);

        Ok(self.writer.to_chunk())
    } 

    fn expression(&mut self) -> Result<()> {
        self.parse_precedence(Precedence::Assignment)
    }

    fn grouping(&mut self) -> Result<()> {
        self.expression()?;
        self.consume(&TokenType::RightParen, "Expected ')'");
        Ok(())
    }


    fn unary(&mut self) -> Result<()> {
        let (prev_token, _) = self.prev()?;
        let operator_type = prev_token.token_type.clone();
        let line = prev_token.line;

        self.parse_precedence(Precedence::Unary);

        match operator_type {
            TokenType::Minus => self.writer.write_op_code(OpCode::Negate, line as i32),
            _ => {}
        }

        Ok(())
    }

    fn binary(&mut self) -> Result<()> {
        let (prev_token, _) = self.prev()?;
        let operator_type = prev_token.token_type.clone();
        let parse_rule = self.get_rule(&operator_type);
        let line = prev_token.line;

        self.parse_precedence((parse_rule.precedence as i32 + 1).into());

        match operator_type {
            TokenType::Plus => self.writer.write_op_code(OpCode::Add, line as i32),
            TokenType::Minus => self.writer.write_op_code(OpCode::Subtract, line as i32),
            TokenType::Star => self.writer.write_op_code(OpCode::Multiply, line as i32),
            TokenType::Slash => self.writer.write_op_code(OpCode::Divide, line as i32),
            _ => {},
        }

        Ok(())
    }

    fn get_rule(&self, operator_type: &TokenType) -> ParseRule {
        todo!()
    }

    fn number(&mut self) -> Result<()> {
        let (token, _) = self.prev()?;
        let num = match token.token_type {
            TokenType::Number(n) => n,
            _ => bail!("Expected token type Num but found some other type")
        };
        self.writer.write_const(num, token.line as i32)
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> Result<()> {
        todo!()
    }

    fn advance(&mut self) {
        self.prev_token = self.current_token.take();

        self.current_token = loop {
            match self.scanner.scan_next()
            {
                Ok(token) => {
                    // println!("Token: {:?}", token);
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

    fn current(&self) -> Result<(&Token, &str)> {
        let current_token = self.current_token.as_ref()
            .context("current token is null")?;
        let lexeme_str = self.lexeme_str(current_token);
        Ok((&current_token, lexeme_str))
    }

    fn prev(&self) -> Result<(&Token, &str)> {
        let prev_token = self.prev_token.as_ref()
            .context("prev token is null")?;
        let lexeme_str = self.lexeme_str(prev_token);
        Ok((&prev_token, lexeme_str))
    }

    fn lexeme_str(&self, token: &Token) -> &str {
        self.scanner.get_lexeme_str(&token.lexeme).expect("Current lexeme out of source boundary")
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

struct ParseRule {
    pub precedence: Precedence
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

#[repr(i32)]
enum Precedence {
  None,
  Assignment,  // =
  Or,          // or
  And,         // and
  Equality,    // == !=
  Comparison,  // < > <= >=
  Term,        // + -
  Factor,      // * /
  Unary,       // ! -
  Call,        // . ()
  Primary
}

impl From<i32> for Precedence {
    fn from(i: i32) -> Self {
        if i > Precedence::Primary as i32 {
            panic!("Failed to convert {} to Precedence", i);
        }
        unsafe { std::mem::transmute(i) }
    }
}