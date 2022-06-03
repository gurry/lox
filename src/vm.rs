use thiserror::Error;

use crate::disassembler::{self, Disassembler};
use crate::instruction::{Instruction, InstructionReader};
use crate::chunk::{Chunk, Value};
use crate::stack::Stack;

type Result<T> = std::result::Result<T, VmError>;

#[derive(Debug)]
pub struct Vm {
    chunk: Chunk,
    stack: Stack<Value>,
    trace: bool
}

impl Vm {
    pub fn new(chunk: Chunk) -> Self {
        Self { chunk, stack: Stack::new(), trace: false }
    }

    pub fn new_with_tracing(chunk: Chunk) -> Self {
        Self { chunk, stack: Stack::new(), trace: true }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut reader = InstructionReader::new(&self.chunk);
        let mut disassembler = Disassembler::new();

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
                            self.stack.push(Value(constant));
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