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
            OpCode::Constant => {
                match instruction.operand1 {
                    Some(index) => {
                        let value = reader.get_const(index as usize)?;
                        println!("{} {:04} '{}'", OpCode::Constant, index, value)
                    }
                    _ => bail!("Opcode {} has no operand", instruction.op_code),
                }
            },
            op_code => println!("{}", op_code)
        };

        Ok(())
    }
}
