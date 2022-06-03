use anyhow::{Result, Context};

use crate::{instruction::{InstructionReader, Instruction, OpCode}, chunk::Chunk};

pub struct Disassembler;

impl Disassembler {
    pub fn disassemble(chunk: &Chunk, name: &str) -> Result<()> {
        println!("== {} ==", name);

        let mut reader = InstructionReader::new(chunk);

        let mut prev_line_number = None;

        loop {
            let read_result =  reader.read_next()
                .with_context(|| "Failed to disassemble instruction")?;

            match read_result {
                Some((instruction, offset, line_number)) => {
                    print!("{:04} ", offset);

                    let same_line_no_as_previous = prev_line_number.is_some() && line_number == prev_line_number.unwrap();
                    if same_line_no_as_previous {
                        print!("   | ");
                    } else {
                        print!("{:4} ", line_number);
                    }

                    prev_line_number = Some(line_number);

                    match instruction {
                        Instruction::Constant(index, value) => println!("{} {:04} '{}'", OpCode::Constant, index, value),
                        Instruction::Return => println!("{}", OpCode::Return),
                    }
                },
                None => break
            }
        }

        Ok(())
    }
}
