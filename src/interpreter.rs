use thiserror::Error;

use crate::instruction::{Instruction, InstructionReader};
use crate::chunk::Chunk;

type Result<T> = std::result::Result<T, InterpreterError>;

#[derive(Debug)]
pub struct Interpreter {
    chunk: Chunk
}

impl Interpreter {
    pub fn new(chunk: Chunk) -> Self {
        Self { chunk }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut reader = InstructionReader::new(&self.chunk);
        loop {
            let read_result =  reader.read_next()
                .map_err(|_| { InterpreterError::new("Failed to read code byte", InterpreterErrorType::CompileError) })?;

            match read_result {
                Some((instruction, _, _)) => {
                    match instruction {
                        Instruction::Constant(_, constant) => println!("{}", constant),
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