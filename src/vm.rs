use anyhow::bail;
use thiserror::Error;

use crate::disassembler::{self, Disassembler};
use crate::instruction::{Instruction, InstructionReader};
use crate::chunk::Chunk;
use crate::stack::Stack;

type Result<T> = std::result::Result<T, VmError>;

#[derive(Debug)]
pub struct Vm {
    chunk: Chunk,
    trace: bool
}

impl Vm {
    pub fn new(chunk: Chunk) -> Self {
        Self { chunk, trace: false }
    }

    pub fn new_with_tracing(chunk: Chunk) -> Self {
        Self { chunk, trace: true }
    }

    pub fn run(&mut self) -> Result<()> {
        let chunk = &self.chunk;
        let mut reader = InstructionReader::new(chunk);
        let mut disassembler = Disassembler::new();
        let mut stack = Stack::new();

        loop {
            let read_result =  reader.read_next()
                .map_err(|_| { VmError::new("Failed to read code byte", VmErrorType::CompileError) })?;

            match read_result {
                Some((instruction, offset, src_line_number)) => {
                    if self.trace {
                        disassembler.disassemble_instruction(&instruction, offset, src_line_number);
                    }

                    match instruction {
                        Instruction::Constant(_, constant) => {
                            println!("{}", constant);
                            stack.push(constant);
                        },
                        Instruction::Return => return Ok(()),
                        Instruction::Negate => {
                            let value = Self::pop(&mut stack)?;
                            stack.push(-value)
                        },
                    }
                },
                None => break
            }
        }

        Ok(())
    }

    fn pop(stack: &mut  Stack<f64>) -> Result<f64> {
        let value = stack.pop()
                .map_err(|_| { VmError::new("Failed to pop stack", VmErrorType::CompileError) })?;
        Ok(value)
    }
}


#[derive(Debug)]
pub enum VmErrorType {
    CompileError,
    RuntimeError
}

#[derive(Error, Debug)]
#[error("{error_type:?}: {msg}")]
pub struct VmError {
    msg: String,
    error_type: VmErrorType
}

impl VmError {
    pub fn new<M: Into<String>>(msg: M, error_type: VmErrorType) -> Self { 
        Self { msg: msg.into(), error_type }
    }
}