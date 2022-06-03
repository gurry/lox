use std::fmt::Display;

use anyhow::{Result, anyhow};

#[derive(Debug)]
pub struct Chunk {
    code: Vec<CodeByte>,
    line_numbers: Vec<i32>,
    constants: Vec<Value>
}

impl Chunk {
    pub fn new() -> Self { 
        Self { code: Vec::new(), line_numbers: Vec::new(), constants: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.code.len()
    }

    pub fn read(&self, offset: usize) -> Result<CodeByte> {
        if offset >= self.code.len() {
            return Err(anyhow!("Offset {} is out range", offset));
        }

        Ok(self.code[offset].clone())
    }

    pub fn get_line_number(&self, offset: usize) -> Result<i32>  {
        if offset >= self.code.len() {
            return Err(anyhow!("Offset {} is out range", offset));
        }

        Ok(self.line_numbers[offset])
    }
    
    pub fn write(&mut self, code_byte: CodeByte, line_number: i32)  {
        self.code.push(code_byte);
        self.line_numbers.push(line_number);
    }

    pub fn add_constant(&mut self, constant: Value) -> u8 {
        self.constants.push(constant);
        (self.constants.len() - 1) as u8
    }

    pub fn get_constant(&self, index: usize) -> Result<Value> {
        if index >= self.constants.len() {
            return Err(anyhow!("Index {} is out range", index));
        }

        Ok(self.constants[index].clone())
    }
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum CodeByte {
    Literal(u8),
    OpCode(OpCode)
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum OpCode {
    Constant,
    Return
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub struct Value(pub f64);

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}