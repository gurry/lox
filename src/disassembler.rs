use anyhow::{Result, Context, bail};

use crate::{instruction::{InstructionReader, Instruction, OpCode}, chunk::Chunk};

pub struct Disassembler {
    prev_src_line_number: Option<i32>
}

impl Disassembler {
    pub fn new() -> Self {
        Self { prev_src_line_number: None }
    }

    pub fn disassemble(&mut self, chunk: &Chunk, name: &str) -> Result<()> {
        println!("== {} ==", name);

        let mut reader = InstructionReader::new(chunk);

        loop {
            let read_result =  reader.read_next()
                .with_context(|| "Failed to disassemble instruction")?;

            match read_result {
                Some((instruction, offset, src_line_number)) => self.disassemble_instruction(&mut reader, &instruction, offset, src_line_number)?,
                None => break
            }
        }

        Ok(())
    }

    pub fn disassemble_instruction<'a>(&mut self, reader: &mut InstructionReader<'a>, instruction: &Instruction, offset: usize, src_line_number: i32) -> Result<()> {
        print!("{:04} ", offset);

        let same_src_line_no_as_prev = self.prev_src_line_number.is_some() && src_line_number == self.prev_src_line_number.unwrap();
        if same_src_line_no_as_prev {
            print!("   | ");
        } else {
            print!("{:4} ", src_line_number);
        }

        self.prev_src_line_number = Some(src_line_number);

        match &instruction.op_code {
            OpCode::Constant | OpCode::DefineGlobal 
            | OpCode::GetGlobal | OpCode::SetGlobal
            | OpCode::GetLocal | OpCode::SetLocal => {
                match instruction.operand1 {
                    Some(operand1) => {
                        print!("{} {:04}", instruction.op_code, operand1);

                        match &instruction.op_code {
                            OpCode::GetLocal | OpCode::SetLocal => {
                                let stack_offset = format!("Stack[{}]", operand1);
                                println!(" '{}'", stack_offset)
                            }
                            _ => {
                                let value = reader.get_const(operand1 as usize)?;
                                println!(" '{}'", value)
                            }
                        }
                    }
                    _ => bail!("Opcode {} has no operand", instruction.op_code),
                }
            },
            OpCode::Jump | OpCode::JumpIfFalse | OpCode::Loop => {
                match (instruction.operand1, instruction.operand2) {
                    (Some(operand1), Some(operand2)) => {
                        println!("{} {:04} {:04}", instruction.op_code, operand1, operand2);
                    }
                    _ => bail!("Opcode {} has one or both operands missing", instruction.op_code),
                }
            },
            op_code => println!("{}", op_code)
        };

        Ok(())
    }
}