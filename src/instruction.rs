use std::fmt::Display;

use crate::chunk::{Chunk, Value};
use anyhow::{Result, bail};

#[derive(Debug)]
pub enum Instruction {
    Constant(i32, f64),
    Return
}

pub struct InstructionWriter<'a> {
    chunk: &'a mut Chunk
}

impl<'a> InstructionWriter<'a> {
    pub fn new(chunk: &'a mut Chunk) -> Self {
        Self { chunk }
    }

    pub fn write_const(&mut self, value: f64, src_line_number: i32) {
        let const_index = self.chunk.add_constant(Value(value));
        self.chunk.write(OpCode::Constant, src_line_number);
        self.chunk.write(const_index, src_line_number);
    }

    pub fn write_return(&mut self, src_line_number: i32)  {
        self.chunk.write(OpCode::Return, src_line_number);
    }
}

pub struct InstructionReader<'a> {
    chunk: &'a Chunk,
    offset: usize
}

impl<'a> InstructionReader<'a> {
    pub fn new(chunk: &'a Chunk) -> Self {
        Self { chunk, offset: 0 }
    }

    pub fn read_next(&mut self) -> Result<Option<(Instruction, usize, i32)>> {
        let code_byte = match self.chunk.read(self.offset) {
            Ok(c) => c,
            Err(_) => return Ok(None),
        };

        let src_line_number = self.chunk.get_src_line_number(self.offset)?;

        let instruction_offset = self.offset;

        self.offset += 1;

        if let Symbol::OpCode(op_code) = code_byte.into() {
            let instruction = match op_code {
                OpCode::Constant => {
                    let const_index = self.chunk.read(self.offset)?;
                    self.offset += 1;
                    let constant = self.chunk.get_constant(const_index as usize)?;
                    Instruction::Constant(const_index as i32, constant.0)
                },
                OpCode::Return => Instruction::Return,
            };
            Ok(Some((instruction, instruction_offset, src_line_number)))
        }
        else {
            bail!("Unknown op code {}", code_byte)
        }
    }
}

#[derive(Debug, Clone)]
pub enum Symbol {
    Literal(u8),
    OpCode(OpCode)
}

impl Into<u8> for Symbol {
    fn into(self) -> u8 {
        match self {
            Symbol::Literal(val) => val,
            Symbol::OpCode(code) => code.into(),
        }
    }
}

impl From<u8> for Symbol {
    fn from(value: u8) -> Self {
        match OpCode::try_from(value) {
            Ok(op_code) => Symbol::OpCode(op_code),
            Err(_) => Symbol::Literal(value),
        }
    }
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum OpCode {
    Constant = 0,
    Return = 1
}

impl Into<u8> for OpCode {
    fn into(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for OpCode {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == OpCode::Constant as u8 => Ok(OpCode::Constant),
            x if x == OpCode::Return as u8 => Ok(OpCode::Return),
            x => bail!("Unknown opcode {}", x),
        }
    }
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}