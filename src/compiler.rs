use core::panic;
use std::{fmt::Display, collections::HashMap, rc::Rc};

use anyhow::{Result, bail, Context, anyhow};
use thiserror::Error;
use crate::{scanner::{Scanner, Token, ScanError, TokenType}, chunk::Chunk, instruction::{OpCode, InstructionWriter}, value::Value};

pub struct Compiler{
    scanner: Scanner,
    writer: InstructionWriter,
    current_token: Option<Token>,
    prev_token: Option<Token>,
    scope_depth: i32,
    locals: Vec<Local>,
    errors: Vec<CompileError>,
    panic_mode: bool,
    parse_rules: ParseRuleTable
}

impl Compiler {
    pub fn new(source: String) -> Self {
        let parse_rules = Self::set_up_parse_rules();
        Self { scanner: Scanner::new(source), writer: InstructionWriter::with_new_chunk(),
            current_token: None, prev_token: None, scope_depth: 0,
            locals: Vec::new(), errors: Vec::new(), panic_mode: false, parse_rules }
    }

    pub fn compile(mut self) -> Result<Chunk> {
        self.advance();

        loop {
            if self.matches(&TokenType::Eof) {
                break
            }

            match self.declaration() {
                Ok(_) => {},
                Err(e) => for err in e.chain().rev() {
                    self.push_current_parse_error(format!("{}", err));
                }
            }
        }

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

    fn declaration(&mut self) -> Result<()> {
        if self.matches(&TokenType::Var) {
            self.var_declaration()?;
        } else {
            self.statement()?;
        }

        if self.panic_mode {
            self.synchronize();
        }

        Ok(())
    }

    fn var_declaration(&mut self) -> Result<()> {
        let global = self.parse_variable("Expected variable name")?;

        if self.matches(&TokenType::Equal) {
            self.expression()?;
        } else {
            let line = self.prev()?.0.line;
            self.writer.write_op_code(OpCode::Nil, line as i32);
        }

        self.consume(&TokenType::Semicolon, "Expected ';' after variable declaration.");

        self.define_variable(global)
    }
    
    fn statement(&mut self) -> Result<()> {
        if self.matches(&TokenType::Print) {
            self.print_statement()?;
        } else if self.matches(&TokenType::LeftBrace) {
            self.begin_scope();
            self.block()?;
            self.end_scope()?;
        } else if self.matches(&TokenType::If) {
            self.if_statement()?;
        } else if self.matches(&TokenType::While) {
            self.while_statement()?;
        } else {
            self.expression_statement()?;
        }

        Ok(())
    }

    fn if_statement(&mut self) -> Result<()> {
        self.consume(&TokenType::LeftParen, "Expected '(' after 'if'.");
        self.expression()?;
        self.consume(&TokenType::RightParen, "Expected ')' after condition"); 


        let line = self.prev()?.0.line;
        let if_jump_addr = self.writer.write_jump_if_false(line as i32);
        self.writer.write_op_code(OpCode::Pop, line as i32); // Pops if expression result

        self.statement()?;

        let else_jump_addr = self.writer.write_jump(line as i32);

        self.writer.patch_jump_to_chunk_end(if_jump_addr)?;
        self.writer.write_op_code(OpCode::Pop, line as i32); // Pops if expression result

        if self.matches(&TokenType::Else) {
            self.statement()?;
        }

        self.writer.patch_jump_to_chunk_end(else_jump_addr)?;

        Ok(())
    }

    fn while_statement(&mut self) -> Result<()> {
        let loop_start = self.writer.len();

        self.consume(&TokenType::LeftParen, "Expected '(' after 'while'.");
        self.expression()?;
        self.consume(&TokenType::RightParen, "Expected ')' after condition"); 


        let line = self.prev()?.0.line;
        let exit_jump_addr = self.writer.write_jump_if_false(line as i32);
        self.writer.write_op_code(OpCode::Pop, line as i32); // Pops if expression result

        self.statement()?;

        self.writer.write_loop(loop_start, line as i32)?;

        self.writer.patch_jump_to_chunk_end(exit_jump_addr)?;
        self.writer.write_op_code(OpCode::Pop, line as i32); // Pops if expression result

        Ok(())
    }

    fn print_statement(&mut self) -> Result<()> {
        self.expression()?;
        self.consume(&TokenType::Semicolon, "Expected ';' after value.");

        let line = self.prev()?.0.line;
        self.writer.write_op_code(OpCode::Print, line as i32);

        Ok(())
    }

    fn block(&mut self) -> Result<()> {
        loop {
            if self.check(&TokenType::RightBrace) || self.check(&TokenType::Eof) {
                break
            }
            self.declaration()?;
        }

        self.consume(&TokenType::RightBrace, "Expected '}' after block");

        Ok(())
    }

    
    fn expression_statement(&mut self) -> Result<()> {
        self.expression()?;
        self.consume(&TokenType::Semicolon, "Expected ';' after expression.");

        let line = self.prev()?.0.line;
        self.writer.write_op_code(OpCode::Pop, line as i32);

        Ok(())
    }

    fn expression(&mut self) -> Result<()> {
        self.parse_precedence(&Precedence::Assignment)
    }

    fn grouping(&mut self, _can_assign: bool) -> Result<()> {
        self.expression()?;
        self.consume(&TokenType::RightParen, "Expected ')'");
        Ok(())
    }

    fn and(&mut self, _can_assign: bool) -> Result<()> { 
        let line = self.prev()?.0.line;
        let end_jump_addr = self.writer.write_jump_if_false(line as i32);
        self.writer.write_op_code(OpCode::Pop, line as i32); // Pops if expression result

        self.parse_precedence(&Precedence::And)?;

        self.writer.patch_jump_to_chunk_end(end_jump_addr)?;

        Ok(())
    }

    fn or(&mut self, _can_assign: bool) -> Result<()> { 
        let line = self.prev()?.0.line;
        let else_jump_addr = self.writer.write_jump_if_false(line as i32);
        let end_jump_addr = self.writer.write_jump(line as i32);

        self.writer.patch_jump_to_chunk_end(else_jump_addr)?;
        self.writer.write_op_code(OpCode::Pop, line as i32); // Pops if expression result

        self.parse_precedence(&Precedence::Or)?;

        self.writer.patch_jump_to_chunk_end(end_jump_addr)?;

        Ok(())
    }

    fn unary(&mut self, _can_assign: bool) -> Result<()> {
        let (prev_token, _) = self.prev()?;
        let operator_type = prev_token.token_type.clone();
        let line = prev_token.line;

        self.parse_precedence(&Precedence::Unary)?;

        match operator_type {
            TokenType::Bang => { self.writer.write_op_code(OpCode::Not, line as i32); },
            TokenType::Minus => { self.writer.write_op_code(OpCode::Negate, line as i32); },
            _ => {}
        };

        Ok(())
    }

    fn binary(&mut self, _can_assign: bool) -> Result<()> {
        let (prev_token, _) = self.prev()?;
        let operator_type = prev_token.token_type.clone();
        let parse_rule = self.get_rule(&operator_type);
        let line = prev_token.line;

        let higher_precedence = parse_rule.precedence.higher();
        self.parse_precedence(&higher_precedence)?;

        match operator_type {
            TokenType::Plus => { self.writer.write_op_code(OpCode::Add, line as i32); },
            TokenType::Minus => { self.writer.write_op_code(OpCode::Subtract, line as i32); },
            TokenType::Star => { self.writer.write_op_code(OpCode::Multiply, line as i32); },
            TokenType::Slash => { self.writer.write_op_code(OpCode::Divide, line as i32); },
            TokenType::BangEqual => {
                self.writer.write_op_code(OpCode::Equal, line as i32);
                self.writer.write_op_code(OpCode::Not, line as i32);
            },
            TokenType::EqualEqual => { self.writer.write_op_code(OpCode::Equal, line as i32); },
            TokenType::Greater => { self.writer.write_op_code(OpCode::Greater, line as i32); },
            TokenType::GreaterEqual => {
                self.writer.write_op_code(OpCode::Less, line as i32);
                self.writer.write_op_code(OpCode::Not, line as i32);
            },
            TokenType::Less => { self.writer.write_op_code(OpCode::Less, line as i32); },
            TokenType::LessEqual => {
                self.writer.write_op_code(OpCode::Greater, line as i32);
                self.writer.write_op_code(OpCode::Not, line as i32);
            },
            _ => {},
        }

        Ok(())
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) -> Result<()> {
        self.scope_depth -= 1;

        if self.locals.len() > 0 {
            let mut i = self.locals.len() - 1;
            loop  {
                if self.locals[i].depth < self.scope_depth {
                    break;
                } 

                let line = self.prev()?.0.line;
                self.writer.write_op_code(OpCode::Pop, line as i32);

                self.locals.pop();

                if i == 0 {
                    break;
                }

                i -= 1;
            }
        }

        Ok(())
    }


    fn variable(&mut self, can_assign: bool) -> Result<()> {
        self.named_variable(self.prev_lexeme_str()?.to_string(), can_assign)
    }

    fn parse_variable(&mut self, msg: &str) -> Result<u8> {
        self.consume(&TokenType::Identifier, msg);

        self.declare_variable()?;
        if self.scope_depth > 0 {
            return Ok(0);
        }

        let c = self.prev_lexeme_str()?.to_string();
        self.identifier_constant(c)
    }

    fn declare_variable(&mut self) -> Result<()> {
        if self.scope_depth == 0 {
            return Ok(());
        }

        let name = self.prev_lexeme_str()?.to_string();

        self.add_local(name);

        Ok(())
    }

    fn add_local(&mut self, name: String) {
        if self.locals.len() >= u8::MAX as usize {
            panic!("Too many locals");
        }
        self.locals.push(Local { name, depth: self.scope_depth, initialized: false });
    }


    fn resolve_local(&self, name: &str) -> Result<Option<i32>> {
        for (i, l) in self.locals.iter().enumerate() {
            if l.name == name {
                if !l.initialized {
                    bail!("Use of uninitialized local variable {}", name);
                }

                return Ok(Some(i as i32));
            }
        }

        Ok(None)
    }

    fn define_variable(&mut self, index: u8) -> Result<()> {
        if self.scope_depth > 0 {
            self.locals.last_mut().unwrap().initialized = true;
            return Ok(());
        }
        let line = self.prev()?.0.line;
        self.writer.write_op_code_with_operand(OpCode::DefineGlobal, index, line as i32);
        Ok(())
    }

    fn identifier_constant(&mut self, s: String) -> Result<u8> {
        Ok(self.writer.add_constant(Value::String(s)))
    }

    fn named_variable(&mut self, name: String, can_assign: bool) -> Result<()> {
        let line = self.prev()?.0.line;

        let (get_op, set_op, operand) = if let Some(local_pos) = self.resolve_local(&name)? {
            (OpCode::GetLocal, OpCode::SetLocal, local_pos as u8)
        } else {
            let index = self.identifier_constant(name)?;
            (OpCode::GetGlobal, OpCode::SetGlobal, index)
        };

        if can_assign && self.matches(&TokenType::Equal) {
            self.expression()?;
            self.writer.write_op_code_with_operand(set_op, operand, line as i32);
        } else {
            self.writer.write_op_code_with_operand(get_op, operand, line as i32);
        }

        Ok(())
    }

    fn number(&mut self, _can_assign: bool) -> Result<()> {
        let (token, lexeme) = self.prev()?;
        let num = lexeme.parse::<f64>()
                .context(format!("Failed to parse '{}' as number", lexeme))?;
        let num = Value::Number(num);
        self.writer.write_const(num, token.line as i32)?;

        Ok(())
    }

    fn string(&mut self, _can_assign: bool) -> Result<()> {
        let (token, lexeme) = self.prev()?;
        let str_copy = lexeme[1..lexeme.len()-1].to_string();
        let str = Value::String(str_copy);
            
        self.writer.write_const(str, token.line as i32)?;

        Ok(())
    }

    fn literal(&mut self, _can_assign: bool) -> Result<()> {
        let (token, _) = self.prev()?;
        match token.token_type {
            TokenType::Nil => { self.writer.write_op_code(OpCode::Nil, token.line as i32); },
            TokenType::True => { self.writer.write_op_code(OpCode::True, token.line as i32); },
            TokenType::False => { self.writer.write_op_code(OpCode::False, token.line as i32); },
            _ => {}
        };

        Ok(())
    }

    fn parse_precedence(&mut self, precedence: &Precedence) -> Result<()> {
        self.advance();

        self.prev_call_prefix(precedence, "Expected expression")?;

        loop {
            let curr_rule = self.current_rule()?;
            if precedence.is_greater_than(&curr_rule.precedence) {
                break;
            }

            self.advance();

            self.prev_call_infix(precedence, "Expected expression")?;
        }

        let can_assign = Precedence::Assignment.is_greater_than(precedence);

        if can_assign && self.matches(&TokenType::Equal) {
            let (token, lexeme) = self.prev()?;
            bail!(CompileError::parse_error("Invalid assignment target", lexeme, token.line))
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

    fn matches(&mut self, token_type: &TokenType) -> bool {
        if !self.check(token_type) {
            return false;
        }

        self.advance();

        true
    }

    fn check(&self, token_type: &TokenType) -> bool {
        match &self.current_token {
            Some(t) => t.token_type == *token_type,
            None => false,
        }
    }

    fn check_prev(&self, token_type: &TokenType) -> bool {
        match &self.prev_token {
            Some(t) => t.token_type == *token_type,
            None => false,
        }
    }

    fn current_rule(&self) -> Result<Rc<ParseRule>> {
        let (current_token, _) = self.current()?;
        Ok(self.get_token_rule(current_token))
    }
 
    fn prev_call_prefix(&mut self, precedence: &Precedence, msg: &str) -> Result<()> {
        let rule = self.prev_rule()?;
        let can_assign = Precedence::Assignment.is_greater_than_or_eq(precedence);
        rule.call_prefix(self, can_assign, msg) 
            .with_context(|| {
                match self.prev() {
                    Ok((token, lexeme)) => anyhow!(CompileError::parse_error(msg, lexeme, token.line)),
                    Err(e) => e,
                }
            })
    }

    fn prev_call_infix(&mut self, precedence: &Precedence, msg: &str) -> Result<()> {
        let rule = self.prev_rule()?;
        let can_assign = Precedence::Assignment.is_greater_than_or_eq(precedence);
        rule.call_infix(self, can_assign, msg) 
            .with_context(|| {
                match self.prev() {
                    Ok((token, lexeme)) => anyhow!(CompileError::parse_error(msg, lexeme, token.line)),
                    Err(e) => e,
                }
            })
    }

    fn prev_rule(&self) -> Result<Rc<ParseRule>> {
        let (prev_token, _) = self.prev()?;
        Ok(self.get_token_rule(prev_token))
    }

    fn get_rule(&self, operator_type: &TokenType) -> Rc<ParseRule> {
        self.parse_rules.get(operator_type)
            .expect(format!("No parse rule found for operator {:?}", operator_type).as_str())
    }

    fn prev_lexeme_str(&self) -> Result<&str> {
        match &self.prev_token {
            Some(t) => Ok(self.lexeme_str(&t)),
            None => bail!("No prev token. Can't get prev lexeme"),
        }
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
        self.push_error(CompileError::parse_error(msg, lexeme, token.line))
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

    fn synchronize(&mut self) {
        self.panic_mode = false;

        loop {
            if self.check(&TokenType::Eof) {
                break;
            }

            if self.check_prev(&TokenType::Semicolon) {
                return;
            }

            match &self.current_token {
                Some(t) => {
                    match t.token_type {
                        TokenType::Class | TokenType::Fun | TokenType::Var | TokenType::For
                        | TokenType::If | TokenType::While | TokenType::Print | TokenType::Return => return,
                        _ => {}
                    };
                },
                _ => {}
            }

            self.advance();
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

        table.add(&TokenType::Bang, Some(Self::unary), None, Precedence::Factor);
        table.add(&TokenType::BangEqual, None, Some(Self::binary), Precedence::Equality);
        table.add_null(&TokenType::Equal);
        table.add(&TokenType::EqualEqual, None, Some(Self::binary), Precedence::Equality);
        table.add(&TokenType::Greater, None, Some(Self::binary), Precedence::Comparison);
        table.add(&TokenType::GreaterEqual, None, Some(Self::binary), Precedence::Comparison);
        table.add(&TokenType::Less, None, Some(Self::binary), Precedence::Comparison);
        table.add(&TokenType::LessEqual, None, Some(Self::binary), Precedence::Comparison);

        table.add(&TokenType::Identifier, Some(Self::variable), None, Precedence::None);
        table.add(&TokenType::String, Some(Self::string), None, Precedence::None);
        table.add(&TokenType::Number, Some(Self::number), None, Precedence::None);


        table.add(&TokenType::And, None, Some(Self::and), Precedence::And);
        table.add_null(&TokenType::Class);
        table.add_null(&TokenType::Else);
        table.add(&TokenType::False, Some(Self::literal), None, Precedence::None);
        table.add_null(&TokenType::Fun);
        table.add_null(&TokenType::For);
        table.add_null(&TokenType::If);
        table.add(&TokenType::Nil, Some(Self::literal), None, Precedence::None);
        table.add(&TokenType::Or, None, Some(Self::or), Precedence::And);
        table.add_null(&TokenType::Print);
        table.add_null(&TokenType::Return);
        table.add_null(&TokenType::Super);
        table.add_null(&TokenType::This);
        table.add(&TokenType::True, Some(Self::literal), None, Precedence::None);
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

type ParseFn = fn(&mut Compiler, bool) -> Result<()>;

struct ParseRule {
    pub prefix: Option<ParseFn>,
    pub infix: Option<ParseFn>,
    pub precedence: Precedence
}

impl ParseRule {
    fn new(prefix: Option<ParseFn>, infix: Option<ParseFn>, precedence: Precedence) -> Self {
        Self { prefix, infix, precedence }
    }

    pub fn call_prefix<M: Into<String>>(&self, c: &mut Compiler, can_assign: bool, msg: M) -> Result<()> {
        Self::call(&self.prefix, c, can_assign, msg)
    }

    pub fn call_infix<M: Into<String>>(&self, c: &mut Compiler, can_assign: bool, msg: M) -> Result<()> {
        Self::call(&self.infix, c, can_assign, msg)
    }

    fn call<M: Into<String>>(callback: &Option<ParseFn>, c: &mut Compiler, can_assign: bool, msg: M) -> Result<()> {
        match callback {
            Some(f) => f(c, can_assign),
            None => bail!(msg.into())
        }
    }
}



#[derive(Clone, Debug, Eq, PartialEq)]
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

    pub fn is_greater_than(&self, other: &Precedence) -> bool {
        self.clone() as i32 > other.clone() as i32
    }

    pub fn is_greater_than_or_eq(&self, other: &Precedence) -> bool {
        self == other || self.is_greater_than(other)
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

#[derive(Clone, Debug)]
struct Local {
    name: String,
    depth: i32,
    initialized: bool
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
    pub fn parse_error<M: Into<String>, N: Into<String>>(msg: M, lexeme: N, line:usize) -> Self { 
        Self::Parse { msg: msg.into(), lexeme: lexeme.into(), line }
    }
}   

