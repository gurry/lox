use std::fmt::Display;

use anyhow::{Result, anyhow};

#[derive(Debug)]
pub struct Chunk {
    code: Vec<u8>,
    src_line_numbers: Vec<i32>,
    constants: Vec<Value>
}

impl Chunk {
    pub fn new() -> Self { 
        Self { code: Vec::new(), src_line_numbers: Vec::new(), constants: Vec::new() }
    }

    pub fn read(&self, offset: usize) -> Result<u8> {
        if offset >= self.code.len() {
            return Err(anyhow!("Offset {} is out range", offset));
        }

        Ok(self.code[offset].clone())
    }

    pub fn get_src_line_number(&self, offset: usize) -> Result<i32>  {
        if offset >= self.code.len() {
            return Err(anyhow!("Offset {} is out range", offset));
        }

        Ok(self.src_line_numbers[offset])
    }
    
    pub fn write<B: Into<u8>>(&mut self, code_byte: B, src_line_number: i32)  {
        self.code.push(code_byte.into());
        self.src_line_numbers.push(src_line_number);
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
pub struct Value(pub f64);

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}