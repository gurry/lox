use anyhow::{Result, anyhow, bail};

use crate::value::Value;

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
    
    pub fn write<B: Into<u8>>(&mut self, code_byte: B, src_line_number: i32) -> usize  {
        self.code.push(code_byte.into());
        self.src_line_numbers.push(src_line_number);
        self.code.len() - 1
    }


    pub fn set<B: Into<u8>>(&mut self, loc: usize, code_byte: B) -> Result<()> {
        if loc >= self.code.len() {
            bail!("Chunk overflow");
        }

        self.code[loc] = code_byte.into();

        Ok(())
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

    pub fn len(&self) -> usize {
        self.code.len()
    }
}