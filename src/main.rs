fn main() {
    let mut chunk = Chunk::new();
    chunk.write(OpCode::Return);
    chunk.disassemble("Test chunk");
}


#[derive(Debug, Clone)]
#[repr(u8)]
enum OpCode {
    Return
}

#[derive(Debug)]
struct Chunk {
    code: Vec<OpCode>
}

impl Chunk {
    fn new() -> Self { 
        Self { code: Vec::new() }
    }

    fn write(&mut self, op_code: OpCode)  {
        self.code.push(op_code);
    }

    fn disassemble(&self, name: &str) {
        println!("== {} ==", name);

        let mut offset = 0;
        loop {
            if offset >= self.code.len() {
                break
            }
            offset = self.disassemble_instruction(offset);
        }
    }

    fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{:04} ", offset);

        let instruction = &self.code[offset];
        match instruction {
            OpCode::Return => self.simple_instruction(instruction, offset),
            _ => {
                println!("Unknown opcode {:?}", instruction);
                offset + 1
            }
        }
    }

    pub fn simple_instruction(&self, op_code: &OpCode, offset: usize) -> usize {
        println!("{:?}", op_code);
        offset + 1
    }
}
