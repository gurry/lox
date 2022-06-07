use std::fmt::Display;

use anyhow::{Context, Result, bail};
use thiserror::Error;

use crate::disassembler::Disassembler;
use crate::instruction::{InstructionReader, OpCode, Instruction};
use crate::chunk::Chunk;
use crate::stack::Stack;
use crate::value::Value;

#[derive(Debug)]
pub struct Vm {
    stack: Stack<Value>,
    trace: bool
}

impl Vm {
    pub fn new(trace: bool) -> Self {
        Self { stack: Stack::new(), trace }
    }

    pub fn run(&mut self, chunk: &mut Chunk) -> Result<()> {
        let mut reader = InstructionReader::new(chunk);
        let mut disassembler = Disassembler::new();
        loop {
            let read_result =  reader.read_next()
            .context(VmError::from_msg("Failed to read code byte"))?;

            match read_result {
                Some((instruction, offset, src_line_number)) => {
                    if self.trace {
                        println!("{:?}", self.stack);
                        disassembler.disassemble_instruction(&mut reader, &instruction, offset, src_line_number)
                            .context(VmError::new("Failed to disassemble instruction", (instruction.clone(), offset, src_line_number)))?;
                    }

                    match instruction.op_code {
                        OpCode::Constant => {
                            match instruction.operand1 {
                                Some(index) => {
                                    let value = reader.get_const(index as usize)
                                        .context(VmError::new(format!("Failed to get constant at index {}", index), (instruction.clone(), offset, src_line_number)))?;
                                    if self.trace {
                                        println!("--> Const: {}", value);
                                    }
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
                            let negated_value = match self.stack.pop()? {
                                Value::Number(n) => Value::Number(-n),
                                _ => bail!("Attempt to negate a non-numeric value")
                            };

                            self.stack.push(negated_value)
                        },
                        OpCode::Add => self.binary_op(|a, b| a + b)?,
                        OpCode::Subtract => self.binary_op(|a, b| a - b)?,
                        OpCode::Multiply => self.binary_op(|a, b| a * b)?,
                        OpCode::Divide => self.binary_op(|a, b| a / b)?,
                        OpCode::Nil => self.stack.push(Value::Nil),
                        OpCode::True => self.stack.push(Value::Boolean(true)),
                        OpCode::False => self.stack.push(Value::Boolean(false)),
                    }
                },
                None => break
            }
        }

        Ok(())
    }

    fn binary_op<O: FnOnce(f64, f64) -> f64>(&mut self, op: O) -> Result<()> {
        let b = self.stack.pop()?;
        let a = self.stack.pop()?;

        let res = match (a, b) {
            (Value::Number(a), Value::Number(b)) => Value::Number(op(a, b)),
            _ => bail!("Attempted binary arithmetic operation on non-numeric operand")
        };

        self.stack.push(res);

        Ok(())
    }
}

#[derive(Error, Debug)]
pub struct VmError {
    msg: String,
    details: Option<(Instruction, usize, i32)>
}

impl VmError {
    pub fn new<M: Into<String>>(msg: M, details: (Instruction, usize, i32)) -> Self { 
        Self { msg: msg.into(), details: Some(details) }
    }


    pub fn from_msg<M: Into<String>>(msg: M) -> Self { 
        Self { msg: msg.into(), details: None }
    }
}

impl Display for VmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.details {
            Some(details) => write!(f, "[source line {}, byte code offset {}, inst '{}'] {}", details.2, details.1, details.0, self.msg),
            None => write!(f, "{}", self.msg),
        }
    }
}