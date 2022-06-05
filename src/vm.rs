use anyhow::{Context, Result, bail};
use thiserror::Error;

use crate::disassembler::Disassembler;
use crate::instruction::{InstructionReader, OpCode};
use crate::chunk::Chunk;
use crate::stack::Stack;

#[derive(Debug)]
pub struct Vm {
    stack: Stack<f64>,
    trace: bool
}

impl Vm {
    pub fn new(trace: bool) -> Self {
        Self { stack: Stack::new(), trace }
    }

    pub fn new_with_tracing() -> Self {
        Self::new(true)
    }

    pub fn run(&mut self, chunk: &mut Chunk) -> Result<()> {
        let mut reader = InstructionReader::new(chunk);
        let mut disassembler = Disassembler::new();
        loop {
            let read_result =  reader.read_next()
            .context(VmError::runtime("Failed to read code byte"))?;

            match read_result {
                Some((instruction, offset, src_line_number)) => {
                    if self.trace {
                        println!("{:?}", self.stack);
                        disassembler.disassemble_instruction(&mut reader, &instruction, offset, src_line_number)
                            .context(VmError::runtime("Failed to disassemble instruction"))?;
                    }

                    match instruction.op_code {
                        OpCode::Constant => {
                            match instruction.operand1 {
                                Some(index) => {
                                    let value = reader.get_const(index as usize)
                                        .context(VmError::runtime(format!("Failed to get constant at index {}", index)))?;
                                    println!("{}", value);
                                    self.stack.push(value);
                                },
                                None => bail!("Opcode {} has no operand", instruction.op_code),
                            }
                        },
                        OpCode::Return => {
                            println!("{}", self.stack.pop()?);
                            return Ok(())
                        },
                        OpCode::Negate => {
                            let value = self.stack.pop()?;
                            self.stack.push(-value)
                        },
                        OpCode::Add => self.binary_op(|a, b| a + b)?,
                        OpCode::Subtract => self.binary_op(|a, b| a - b)?,
                        OpCode::Multiply => self.binary_op(|a, b| a * b)?,
                        OpCode::Divide => self.binary_op(|a, b| a / b)?,
                    }
                },
                None => break
            }
        }

        Ok(())
    }

    fn binary_op<O: FnOnce(f64, f64) -> f64>(&mut self, op: O) -> Result<()> {
        let a = self.stack.pop()?;
        let b = self.stack.pop()?;
        let res = op(a, b);
        self.stack.push(res);

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

    pub fn compile<M: Into<String>>(msg: M) -> Self { 
        Self::new(msg, VmErrorType::CompileError)
    }

    pub fn runtime<M: Into<String>>(msg: M) -> Self { 
        Self::new(msg, VmErrorType::RuntimeError)
    }
}