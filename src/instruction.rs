use std::fmt::Display;

use crate::{chunk::Chunk, value::Value};
use anyhow::{Result, bail};

#[derive(Debug, Clone)]
pub struct Instruction {
    pub op_code: OpCode,
    pub operand1: Option<u8>,
    pub operand2: Option<u8>
}

impl Instruction {
    pub fn new(op_code: OpCode, operand1: Option<u8>, operand2: Option<u8>) -> Self {
        Self { op_code, operand1, operand2 }
    }

    pub fn simple(op_code: OpCode) -> Self {
        Self::new(op_code, None, None)
    }

    pub fn unary(op_code: OpCode, operand: u8) -> Self {
        Self::new(op_code, Some(operand), None)
    }

    pub fn binary(op_code: OpCode, operand1: u8, operand2: u8) -> Self {
        Self::new(op_code, Some(operand1), Some(operand2))
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.op_code)?;
        match self.operand1 {
            Some(o) => write!(f, " {}", o)?,
            None => {},
        };

        match self.operand2 {
            Some(o) => write!(f, " {}", o),
            None => Ok(()),
        }
    }
}

pub struct InstructionWriter {
    chunk: Chunk
}

impl InstructionWriter {
    pub fn new(chunk: Chunk) -> Self {
        Self { chunk }
    }

    pub fn with_new_chunk() -> Self {
        Self { chunk: Chunk::new() }
    }

    pub fn to_chunk(self) -> Chunk {
        self.chunk
    }

    pub fn write_const(&mut self, value: Value, src_line_number: i32) -> Result<()> {
        let const_index = self.chunk.add_constant(value);
        if const_index > u8::MAX {
            bail!("Too many costants in chunk")
        }
        self.chunk.write(OpCode::Constant, src_line_number);
        self.chunk.write(const_index, src_line_number);

        Ok(())
    }

    pub fn write_op_code<I: Into<i32>>(&mut self, op_code: OpCode, src_line_number: I)  {
        self.chunk.write(op_code, src_line_number.into());
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

        let op_code: OpCode = code_byte.try_into()?;

        let instruction = match op_code {
            OpCode::Constant => {
                let const_index = self.chunk.read(self.offset)?;
                self.offset += 1;
                Instruction::unary(OpCode::Constant, const_index)
            },
            op_code => Instruction::simple(op_code)
        };
        Ok(Some((instruction, instruction_offset, src_line_number)))
    }


    pub fn get_const(&self, index: usize) -> Result<Value> {
        self.chunk.get_constant(index)
    }
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum OpCode {
    Constant = 0,
    Return = 1,
    Negate = 2,
    Add = 3,
    Subtract = 4,
    Multiply = 5,
    Divide = 6,
    Nil = 7,
    True = 8,
    False = 9
}

impl Into<u8> for OpCode {
    fn into(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for OpCode {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > OpCode::False as u8 {
            bail!("Unknown opcode {}", value);
        }

        Ok(unsafe { std::mem::transmute(value) })
    }
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
