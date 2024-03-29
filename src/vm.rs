use std::collections::HashMap;
use std::fmt::Display;

use anyhow::{Context, Result, bail, anyhow};
use thiserror::Error;

use crate::disassembler::Disassembler;
use crate::instruction::{InstructionReader, OpCode, Instruction};
use crate::chunk::Chunk;
use crate::stack::Stack;
use crate::value::Value;

#[derive(Debug)]
pub struct Vm {
    stack: Stack<Value>,
    globals: HashMap<String, Value>,
    trace: bool
}

impl Vm {
    pub fn new(trace: bool) -> Self {
        Self { stack: Stack::new(), globals: HashMap::new(), trace }
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
                        OpCode::Print => println!("{}", self.stack.pop()?),
                        OpCode::Pop => { let _ = self.stack.pop()?; },
                        OpCode::DefineGlobal => {
                            let global_name = self.get_global_name(&instruction, &reader)?;

                            let val = self.stack.peek(0)?;
                            self.globals.insert(global_name, val.clone());
                            self.stack.pop()?;
                        },
                        OpCode::GetGlobal => {
                            let val =  self.get_global(&instruction, &reader)?;
                            self.stack.push(val);
                        },
                        OpCode::SetGlobal => {
                            let global_name = self.get_global_name(&instruction, &reader)?;
                            
                            if !self.globals.contains_key(&global_name) {
                                bail!(VmError::from_msg(format!("Undefined variable '{}'", global_name)));
                            }

                            let new_value = self.stack.peek(0)?.clone();
                            self.globals.insert(global_name, new_value);
                        },
                        OpCode::GetLocal => {
                            let slot = Self::get_operand1(&instruction)?;
                            let val = self.stack.peek_front( slot as usize)?;
                            self.stack.push(val.clone());
                        },
                        OpCode::SetLocal => {
                            let slot = Self::get_operand1(&instruction)?;
                            let val = self.stack.peek(0)?;
                            self.stack.set_front(slot as usize, val.clone())?;
                        },
                        OpCode::Jump => {
                            let jmp_offset = Self::read_operands_as_usize(instruction)?;
                            reader.inc_ip(jmp_offset)?;
                        }
                        OpCode::JumpIfFalse => {
                            let jmp_offset = Self::read_operands_as_usize(instruction)?;
                            match self.stack.peek(0)? {
                                Value::Boolean(v) => if !*v {
                                    reader.inc_ip(jmp_offset)?;
                                },
                                _ => bail!("Can't jump. Non boolean value found on stack")
                            };
                        },
                        OpCode::Loop => {
                            let jmp_offset = Self::read_operands_as_usize(instruction)?;
                            reader.dec_ip(jmp_offset)?;
                        },
                    }
                },
                None => break
            }
        }

        Ok(())
    }

    fn get_global(&mut self, instruction: &Instruction, reader: &InstructionReader) -> Result<Value> {
        let global_name = self.get_global_name(&instruction, &reader)?;

        match self.globals.get(&global_name) {
            Some(v) => Ok(v.clone()),
            None => bail!(VmError::from_msg(format!("Undefined variable '{}'", global_name))),
        }
    }

    fn get_global_name(&mut self, instruction: &Instruction, reader: &InstructionReader) -> Result<String> {
        let global_name_index = Self::get_operand1(instruction)?;

        let constant = reader.get_const(global_name_index as _)
            .context(anyhow!("No global at index {}", global_name_index))?;
        
        match constant {
            Value::String(name) => Ok(name),
            _ => bail!(VmError::from_msg(format!("Operand 1 missing on instruction {}", instruction.op_code)))
        }
    }

    fn get_operand1(instruction: &Instruction) -> Result<u8> {
        instruction.operand1
            .ok_or(anyhow!(VmError::from_msg(format!("Operand 1 missing on instruction {}", instruction.op_code))))
    }

    fn get_operand2(instruction: &Instruction) -> Result<u8> {
        instruction.operand2
            .ok_or(anyhow!(VmError::from_msg(format!("Operand 2 missing on instruction {}", instruction.op_code))))
    }

    fn read_operands_as_usize(instruction: Instruction) -> Result<usize, anyhow::Error> {
        let op1 = Self::get_operand1(&instruction)? as usize;
        let op2 = Self::get_operand2(&instruction)? as usize;
        let jmp_offset = op1 << 8 | op2;
        Ok(jmp_offset)
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