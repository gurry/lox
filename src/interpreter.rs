use thiserror::Error;

use crate::chunk::{Chunk, CodeByte, OpCode};

type Result<T> = std::result::Result<T, InterpreterError>;

#[derive(Debug)]
pub struct Interpreter {
    chunk: Chunk,
    ip: usize
}

impl Interpreter {
    pub fn new(chunk: Chunk) -> Self {
        Self { chunk, ip: 0 }
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            let code_byte = self.chunk.read_code_byte(self.ip)
                .map_err(|_| { InterpreterError::new(format!("Failed to read code byte at ip {}", self.ip), InterpreterErrorType::CompileError) })?;
            self.ip += 1;

            match code_byte {
                CodeByte::Literal(_) => {},
                CodeByte::OpCode(op_code) => {
                    match op_code {
                        OpCode::Constant => {},
                        OpCode::Return => return Ok(()),
                    }
                },
            }
        }
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