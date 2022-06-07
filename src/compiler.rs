use core::panic;
use std::{fmt::Display, collections::HashMap, rc::Rc};

use anyhow::{Result, bail, Context};
use thiserror::Error;
use crate::{scanner::{Scanner, Token, ScanError, TokenType}, chunk::Chunk, instruction::{OpCode, InstructionWriter}};

pub struct Compiler{
    scanner: Scanner,
    writer: InstructionWriter,
    current_token: Option<Token>,
    prev_token: Option<Token>,
    errors: Vec<CompileError>,
    panic_mode: bool,
    parse_rules: ParseRuleTable
}

impl Compiler {
    pub fn new(source: String) -> Self {
        let parse_rules = Self::set_up_parse_rules();
        Self { scanner: Scanner::new(source), writer: InstructionWriter::with_new_chunk(), current_token: None, prev_token: None, errors: Vec::new(), panic_mode: false, parse_rules }
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
        self.parse_precedence(&Precedence::Assignment)
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

        self.parse_precedence(&Precedence::Unary)?;

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

        let higher_precedence = parse_rule.precedence.higher();
        self.parse_precedence(&higher_precedence)?;

        match operator_type {
            TokenType::Plus => self.writer.write_op_code(OpCode::Add, line as i32),
            TokenType::Minus => self.writer.write_op_code(OpCode::Subtract, line as i32),
            TokenType::Star => self.writer.write_op_code(OpCode::Multiply, line as i32),
            TokenType::Slash => self.writer.write_op_code(OpCode::Divide, line as i32),
            _ => {},
        }

        Ok(())
    }

    fn get_rule(&self, operator_type: &TokenType) -> Rc<ParseRule> {
        self.parse_rules.get(operator_type)
            .expect(format!("No parse rule found for operator {:?}", operator_type).as_str())
    }

    fn number(&mut self) -> Result<()> {
        let (token, lexeme) = self.prev()?;
        let num = lexeme.parse::<f64>()
                .context(format!("Failed to parse '{}' as number", lexeme))?;
        self.writer.write_const(num, token.line as i32)
    }

    fn parse_precedence(&mut self, precedence: &Precedence) -> Result<()> {
        self.advance();

        self.prev_rule()?.call_prefix(self,"Expected expression")?;

        loop {
            let curr_rule = self.current_rule()?;
            if precedence.is_greater_then(&curr_rule.precedence) {
                break;
            }

            self.advance();

            self.prev_rule()?.call_infix(self,"Expected expression")?;
        }

        Ok(())
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

    fn current_rule(&self) -> Result<Rc<ParseRule>> {
        let (current_token, _) = self.current()?;
        Ok(self.get_token_rule(current_token))
    }
 
    fn prev_rule(&self) -> Result<Rc<ParseRule>> {
        let (prev_token, _) = self.prev()?;
        Ok(self.get_token_rule(prev_token))
    }

    fn get_token_rule(&self, token: &Token) -> Rc<ParseRule> {
        let operator_type = token.token_type.clone();
        self.get_rule(&operator_type)
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

    fn set_up_parse_rules() -> ParseRuleTable {
        let mut table = ParseRuleTable::new();

        table.add(&TokenType::LeftParen, Some(Self::grouping), None, Precedence::None);
        table.add_null(&TokenType::RightParen);
        table.add_null(&TokenType::LeftBrace);
        table.add_null(&TokenType::RightBrace);
        table.add_null(&TokenType::Comma);
        table.add_null(&TokenType::Dot);
        table.add(&TokenType::Minus, Some(Self::unary), Some(Self::binary), Precedence::Term);
        table.add(&TokenType::Plus, None, Some(Self::binary), Precedence::Term);
        table.add_null(&TokenType::Semicolon);
        table.add(&TokenType::Slash, None, Some(Self::binary), Precedence::Factor);
        table.add(&TokenType::Star, None, Some(Self::binary), Precedence::Factor);

        table.add_null(&TokenType::Bang);
        table.add_null(&TokenType::BangEqual);
        table.add_null(&TokenType::Equal);
        table.add_null(&TokenType::EqualEqual);
        table.add_null(&TokenType::Greater);
        table.add_null(&TokenType::GreaterEqual);
        table.add_null(&TokenType::Less);
        table.add_null(&TokenType::LessEqual);

        table.add_null(&TokenType::Identifier);
        table.add(&TokenType::Number, Some(Self::number), None, Precedence::None);


        table.add_null(&TokenType::And);
        table.add_null(&TokenType::Class);
        table.add_null(&TokenType::Else);
        table.add_null(&TokenType::False);
        table.add_null(&TokenType::Fun);
        table.add_null(&TokenType::For);
        table.add_null(&TokenType::If);
        table.add_null(&TokenType::Nil);
        table.add_null(&TokenType::Or);
        table.add_null(&TokenType::Print);
        table.add_null(&TokenType::Return);
        table.add_null(&TokenType::Super);
        table.add_null(&TokenType::This);
        table.add_null(&TokenType::True);
        table.add_null(&TokenType::Var);
        table.add_null(&TokenType::While);

        table.add_null(&TokenType::Eof);

        table
    } 
}

struct ParseRuleTable {
    lookup: HashMap<TokenType, Rc<ParseRule>> 
}

impl ParseRuleTable {
    pub fn new() -> Self {
        Self { lookup: HashMap::new() }
    }

    pub fn add(&mut self, token_type: &TokenType, prefix: Option<ParseFn>, infix: Option<ParseFn>, precedence: Precedence) {
        self.lookup.insert(token_type.clone(), Rc::new(ParseRule::new(prefix, infix, precedence)));
    }

    pub fn add_null(&mut self, token_type: &TokenType) {
        self.add(token_type, None, None,Precedence::None)
    }

    pub fn get(&self, token_type: &TokenType) -> Option<Rc<ParseRule>> {
       self.lookup.get(token_type).map(|p| p.clone())
    }
}

type ParseFn = fn(&mut Compiler) -> Result<()>;

struct ParseRule {
    pub prefix: Option<ParseFn>,
    pub infix: Option<ParseFn>,
    pub precedence: Precedence
}

impl ParseRule {
    fn new(prefix: Option<ParseFn>, infix: Option<ParseFn>, precedence: Precedence) -> Self {
        Self { prefix, infix, precedence }
    }

    pub fn call_prefix<M: Into<String>>(&self, c: &mut Compiler, msg: M) -> Result<()> {
        Self::call(&self.prefix, c, msg)
    }

    pub fn call_infix<M: Into<String>>(&self, c: &mut Compiler, msg: M) -> Result<()> {
        Self::call(&self.infix, c, msg)
    }

    fn call<M: Into<String>>(callback: &Option<ParseFn>, c: &mut Compiler, msg: M) -> Result<()> {
        match callback {
            Some(f) => f(c),
            None => bail!(msg.into())
        }
    }
}



#[derive(Clone, Debug)]
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

impl Precedence {
    pub fn higher(&self) -> Precedence {
        let clone = self.clone();
        (clone as i32 + 1).into()
    }

    pub fn is_greater_then(&self, other: &Precedence) -> bool {
        self.clone() as i32 > other.clone() as i32
    }
}

impl From<i32> for Precedence {
    fn from(i: i32) -> Self {
        if i > Precedence::Primary as i32 {
            panic!("Failed to convert {} to Precedence", i);
        }
        unsafe { std::mem::transmute(i) }
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

