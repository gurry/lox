use anyhow::{Context, Result};
use thiserror::Error;

use crate::disassembler::Disassembler;
use crate::instruction::{Instruction, InstructionReader};
use crate::chunk::Chunk;
use crate::stack::Stack;

#[derive(Debug)]
pub struct Vm {
    chunk: Chunk,
    trace: bool
}

impl Vm {
    pub fn new(chunk: Chunk, trace: bool) -> Self {
        Self { chunk, trace }
    }

    pub fn new_with_tracing(chunk: Chunk) -> Self {
        Self::new(chunk, true)
    }

    pub fn run(&mut self) -> Result<()> {
        let chunk = &self.chunk;
        let mut reader = InstructionReader::new(chunk);
        let mut disassembler = Disassembler::new();
        let mut stack = Stack::new();
        loop {
            let read_result =  reader.read_next()
            .context(VmError::runtime("Failed to read code byte"))?;

            match read_result {
                Some((instruction, offset, src_line_number)) => {
                    if self.trace {
                        disassembler.disassemble_instruction(&mut reader, &instruction, offset, src_line_number)
                            .context(VmError::runtime("Failed to disassemble instruction"))?;
                    }

                    match instruction {
                        Instruction::Constant(index) => {
                            let value = reader.get_const(index as usize)
                                .context(VmError::runtime(format!("Failed to get constant at index {}", index)))?;
                            println!("{}", value);
                            stack.push(value);
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
                .context(VmError::runtime("Failed to pop stack"))?;
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

    pub fn compile<M: Into<String>>(msg: M) -> Self { 
        Self::new(msg, VmErrorType::CompileError)
    }

    pub fn runtime<M: Into<String>>(msg: M) -> Self { 
        Self::new(msg, VmErrorType::RuntimeError)
    }
}