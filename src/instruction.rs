use crate::chunk::{CodeByte, Chunk, Value, OpCode};
use anyhow::Result;

#[derive(Debug)]
pub enum Instruction {
    Constant(f64),
    Return
}

pub struct InstructionWriter<'a> {
    chunk: &'a mut Chunk
}

impl<'a> InstructionWriter<'a> {
    pub fn new(chunk: &'a mut Chunk) -> Self {
        Self { chunk }
    }

    pub fn write(&mut self, instruction: Instruction, line_number: i32)  {
        match instruction {
            Instruction::Constant(const_value) => {
                let const_index = self.chunk.add_constant(Value(const_value));
                self.chunk.write(CodeByte::OpCode(OpCode::Constant), line_number);
                self.chunk.write(CodeByte::Literal(const_index), line_number);
            },
            Instruction::Return => self.chunk.write(CodeByte::OpCode(OpCode::Return), line_number),
        }
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

    pub fn read_next(&mut self) -> Result<Option<Instruction>> {
        todo!()
    }
}



