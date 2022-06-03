use crate::chunk::{Chunk, OpCode, CodeByte};

use anyhow::Result;

pub struct Disassembler;

impl Disassembler {
    pub fn disassemble(chunk: &Chunk, name: &str) -> Result<()> {
        println!("== {} ==", name);

        let mut offset = 0;
        loop {
            if offset >= chunk.len() {
                break
            }
            offset = Self::disassemble_instruction(chunk, offset)?;
        }

        Ok(())
    }

    fn disassemble_instruction(chunk: &Chunk, offset: usize) -> Result<usize> {
        print!("{:04} ", offset);

        let prev_line_number = if offset > 0 { Some(chunk.get_line_number(offset - 1)?) } else { None };
        let line_number = chunk.get_line_number(offset)?;
        let same_line_no_as_previous = prev_line_number.is_some() && line_number == prev_line_number.unwrap();
        if same_line_no_as_previous {
            print!("   | ");
        } else {
            print!("{:4} ", line_number);
        }

        let instruction = chunk.read(offset)?;
        let next_offset = match instruction {
            CodeByte::OpCode(op_code) => {
                match op_code {
                    OpCode::Return => Self::simple_instruction(&op_code, offset),
                    OpCode::Constant => Self::constant_instruction(chunk, offset)?
                }
            },
            _ => {
                println!("Unknown opcode {:?}", instruction);
                offset + 1
            }
        };

        Ok(next_offset)
    }

    fn simple_instruction(op_code: &OpCode, offset: usize) -> usize {
        println!("{}", op_code);
        offset + 1
    }

    fn constant_instruction(chunk: &Chunk, offset: usize) -> Result<usize> {
        let constant_offset = offset + 1;
        if let CodeByte::Literal(constant) = chunk.read(constant_offset)? {
            println!("{} {:04} '{}'", OpCode::Constant, constant, chunk.get_constant(constant as usize)?);
        }
        else {
            println!("Expected constant literal at offset {} but found unknown code", constant_offset);
        }

        Ok(constant_offset + 1)
    }
}
