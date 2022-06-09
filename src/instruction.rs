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

    pub fn len(&self) -> usize {
        self.chunk.len()
    }

    pub fn write_const(&mut self, value: Value, src_line_number: i32) -> Result<usize> {
        let const_index = self.chunk.add_constant(value);
        if const_index > u8::MAX {
            bail!("Too many costants in chunk")
        }
        let start = self.chunk.write(OpCode::Constant, src_line_number);
        self.chunk.write(const_index, src_line_number);

        Ok(start)
    }

    pub fn write_op_code_with_operand(&mut self, op_code: OpCode, operand: u8, src_line_number: i32) -> usize {
        let start = self.chunk.write(op_code, src_line_number);
        self.chunk.write(operand, src_line_number);
        start
    }

    pub fn write_op_code_with_operands(&mut self, op_code: OpCode, operand1: u8, operand2: u8, src_line_number: i32) -> usize {
        let start = self.chunk.write(op_code, src_line_number);
        self.chunk.write(operand1, src_line_number);
        self.chunk.write(operand2, src_line_number);
        start
    }

    pub fn write_op_code<I: Into<i32>>(&mut self, op_code: OpCode, src_line_number: I) -> usize  {
        self.chunk.write(op_code, src_line_number.into())
    }

    pub fn write_jump_if_false(&mut self, src_line_number: i32) -> usize {
        self.write_op_code_with_operands(OpCode::JumpIfFalse, 0xff,0xff, src_line_number)
    }

    pub fn write_jump(&mut self, src_line_number: i32) -> usize {
        self.write_op_code_with_operands(OpCode::Jump, 0xff,0xff, src_line_number)
    }

    pub fn write_loop(&mut self, loop_start_loc: usize, src_line_number: i32) -> Result<usize> {
        let offset = self.chunk.len() - (loop_start_loc - 3);

        if offset > usize::MAX {
            bail!("Loop body too big ({})", offset);
        }

        let op1 = ((offset >> 8) & 0xff) as u8;
        let op2 = (offset & 0xff) as u8;
        let start = self.write_op_code_with_operands(OpCode::Loop, op1, op2, src_line_number);

        Ok(start)
    }

    pub fn set_byte(&mut self, loc: usize, code_byte: u8) -> Result<()> {
        self.chunk.set(loc, code_byte)
    }

    pub fn patch_operands(&mut self, op_code_loc: usize, operand1: Option<u8>, operand2: Option<u8>) -> Result<()> {
        if let Some(op1) = operand1 {
            self.set_byte(op_code_loc + 1, op1)?;
        }

        if let Some(op2) = operand2 {
            self.set_byte(op_code_loc + 2, op2)?;
        }

        Ok(())
    }

    pub fn patch_jump_to_chunk_end(&mut self, jmp_op_code_loc: usize) -> Result<()> {
        let relative_offset_to_current_chunk_end = self.chunk.len() - (jmp_op_code_loc + 3);

        if relative_offset_to_current_chunk_end > usize::MAX {
            bail!("Jump too long ({})", relative_offset_to_current_chunk_end);
        }

        let operand1 = (relative_offset_to_current_chunk_end >> 8) & 0xff;
        let operand2 = relative_offset_to_current_chunk_end & 0xff;

        self.patch_operands(jmp_op_code_loc, Some(operand1 as u8), Some(operand2 as u8))?;

        Ok(())
    }

    pub fn add_constant(&mut self, value: Value) -> u8 { 
        self.chunk.add_constant(value)
    }
}

pub struct InstructionReader<'a> {
    chunk: &'a Chunk,
    ip: usize
}

impl<'a> InstructionReader<'a> {
    pub fn new(chunk: &'a Chunk) -> Self {
        Self { chunk, ip: 0 }
    }

    pub fn read_next(&mut self) -> Result<Option<(Instruction, usize, i32)>> {
        let code_byte = match self.chunk.read(self.ip) {
            Ok(c) => c,
            Err(_) => return Ok(None),
        };

        let src_line_number = self.chunk.get_src_line_number(self.ip)?;

        let instruction_offset = self.ip;

        self.ip += 1;

        let op_code: OpCode = code_byte.try_into()?;

        let instruction = match op_code {
            OpCode::Constant | OpCode::DefineGlobal
            | OpCode::GetGlobal | OpCode::SetGlobal 
            | OpCode::GetLocal | OpCode::SetLocal => {
                let operand1 = self.chunk.read(self.ip)?;
                self.ip += 1;
                Instruction::unary(op_code, operand1)
            },
            OpCode::Jump | OpCode::JumpIfFalse | OpCode::Loop  => {
                let operand1 = self.chunk.read(self.ip)?;
                self.ip += 1;
                let operand2 = self.chunk.read(self.ip)?;
                self.ip += 1;
                Instruction::binary(op_code, operand1, operand2)
            },
            op_code => Instruction::simple(op_code)
        };
        Ok(Some((instruction, instruction_offset, src_line_number)))
    }


    pub fn get_const(&self, index: usize) -> Result<Value> {
        self.chunk.get_constant(index)
    }

    pub fn set_ip(&mut self, new_ip: usize) -> Result<()> {
        if new_ip > self.chunk.len() {
            bail!("Attempt to set ip beyond chunk ({})", new_ip);
        }

        self.ip = new_ip;

        Ok(())
    }

    pub fn inc_ip(&mut self, inc: usize) -> Result<()> {
        self.set_ip(self.ip + inc)
    }

    pub fn dec_ip(&mut self, dec: usize) -> Result<()> {
        self.set_ip(self.ip - dec)
    }
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum OpCode {
    Constant,
    Return,
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    Nil,
    True,
    False,
    Not,
    Equal,
    Greater,
    Less,
    Print,
    Pop,
    DefineGlobal,
    GetGlobal,
    SetGlobal,
    GetLocal,
    SetLocal,
    Jump,
    JumpIfFalse,
    Loop
}

impl Into<u8> for OpCode {
    fn into(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for OpCode {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > OpCode::Loop as u8 {
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
