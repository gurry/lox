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
                                _ => bail!(VmError::new("Attempt to negate a non-numeric value", (instruction.clone(), offset, src_line_number)))
                            };

                            self.stack.push(negated_value)
                        },
                        OpCode::Add => {
                            let a = self.stack.peek(1)?;
                            let b = self.stack.peek(0)?;

                            match (a, b) {
                                (Value::Number(_), Value::Number(_)) => self.num_binary_op(|a, b| a + b)?,
                                (Value::String(_), Value::String(_)) => self.binary_op(|a, b| {
                                    match (a, b) {
                                    (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                                    _ => bail!("Attempted add or concatenate on non-numeric or non-string operands")
                                } })?,
                                _ => bail!("Attempted add or concatenate on non-numeric or non-string operands")
                            };
                        },
                        OpCode::Subtract => self.num_binary_op(|a, b| a - b)?,
                        OpCode::Multiply => self.num_binary_op(|a, b| a * b)?,
                        OpCode::Divide => self.num_binary_op(|a, b| a / b)?,
                        OpCode::Nil => self.stack.push(Value::Nil),
                        OpCode::True => self.stack.push(Value::Boolean(true)),
                        OpCode::False => self.stack.push(Value::Boolean(false)),
                        OpCode::Not => {
                            match self.stack.pop()? {
                                Value::Boolean(v) => self.stack.push(Value::Boolean(!v)),
                                _ => bail!(VmError::new("Attempted not on a non-bool value", (instruction.clone(), offset, src_line_number)))
                            }
                        },
                        OpCode::Equal => self.binary_op(|a, b| Ok(Value::Boolean(a == b)))?,
                        OpCode::Greater => self.binary_op(|a, b| Ok(Value::Boolean(a > b)))?,
                        OpCode::Less => self.binary_op(|a, b| Ok(Value::Boolean(a < b)))?,
                    }
                },
                None => break
            }
        }

        Ok(())
    }

    fn binary_op<O: FnOnce(&Value, &Value) -> Result<Value>>(&mut self, op: O) -> Result<()> {
        let b = self.stack.pop()?;
        let a = self.stack.pop()?;

        let res = op(&a, &b)?;

        self.stack.push(res);

        Ok(())
    }


    fn num_binary_op<O: FnOnce(f64, f64) -> f64>(&mut self, op: O) -> Result<()> {
        self.binary_op(|a, b| {
            match (a, b) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(op(*a, *b))),
                _ => bail!("Numberic operation attempted on non-numbeic values")
            }
        })
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