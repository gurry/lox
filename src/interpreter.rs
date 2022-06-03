use thiserror::Error;

use crate::disassembler::{self, Disassembler};
use crate::instruction::{Instruction, InstructionReader};
use crate::chunk::Chunk;

type Result<T> = std::result::Result<T, InterpreterError>;

#[derive(Debug)]
pub struct Interpreter {
    chunk: Chunk,
    trace: bool
}

impl Interpreter {
    pub fn new(chunk: Chunk) -> Self {
        Self { chunk, trace: false }
    }

    pub fn new_with_tracing(chunk: Chunk) -> Self {
        Self { chunk, trace: true }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut reader = InstructionReader::new(&self.chunk);
        let mut disassembler = Disassembler::new();

        loop {
            let read_result =  reader.read_next()
                .map_err(|_| { InterpreterError::new("Failed to read code byte", InterpreterErrorType::CompileError) })?;

            match read_result {
                Some((instruction, offset, src_line_number)) => {
                    if self.trace {
                        disassembler.disassemble_instruction(&instruction, offset, src_line_number);
                    }

                    match instruction {
                        Instruction::Constant(_, constant) => {
                            println!("{}", constant)
                        },
                        Instruction::Return => return Ok(()),
                    }
                },
                None => break
            }
        }

        Ok(())
    }
}


#[derive(Debug)]
pub enum InterpreterErrorType {
    CompileError,
    RuntimeError
}

#[derive(Error, Debug)]
#[error("{error_type:?}: {msg}")]
pub struct InterpreterError {
    msg: String,
    error_type: InterpreterErrorType
}

impl InterpreterError {
    pub fn new<M: Into<String>>(msg: M, error_type: InterpreterErrorType) -> Self { 
        Self { msg: msg.into(), error_type }
    }
}