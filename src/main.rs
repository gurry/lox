use std::fmt::Display;

fn main() {
    let mut chunk = Chunk::new();
    let constant = chunk.add_constant(Value(1.2));
    chunk.write_op_code(OpCode::Constant);
    chunk.write_addr(constant as u8);
    chunk.write_op_code(OpCode::Return);
    chunk.disassemble("Test chunk");
}

#[derive(Debug, Clone)]
#[repr(u8)]
enum CodeByte {
    Addr(u8),
    OpCode(OpCode)
}

#[derive(Debug, Clone)]
#[repr(u8)]
enum OpCode {
    Constant,
    Return
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
struct Chunk {
    code: Vec<CodeByte>,
    values: Vec<Value>
}

impl Chunk {
    fn new() -> Self { 
        Self { code: Vec::new(), values: Vec::new() }
    }

    fn write_op_code(&mut self, op_code: OpCode)  {
        self.write_code(CodeByte::OpCode(op_code))
    }

    fn write_addr(&mut self, addr: u8)  {
        self.write_code(CodeByte::Addr(addr))
    }

    fn write_code(&mut self, code_byte: CodeByte)  {
        self.code.push(code_byte);
    }

    fn add_constant(&mut self, value: Value) -> u8 {
        self.values.push(value);
        (self.values.len() - 1) as u8
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
            CodeByte::OpCode(op_code) => {
                match op_code {
                    OpCode::Return => self.simple_instruction(op_code, offset),
                    OpCode::Constant => self.constant_instruction(offset)
                }
            },
            _ => {
                println!("Unknown opcode {:?}", instruction);
                offset + 1
            }
        }
    }

    fn simple_instruction(&self, op_code: &OpCode, offset: usize) -> usize {
        println!("{}", op_code);
        offset + 1
    }

    fn constant_instruction(&self, offset: usize) -> usize {
        let constant_offset = offset + 1;
        if let CodeByte::Addr(constant) = self.code[constant_offset] {
            println!("{} {:04} '{}'", OpCode::Constant, constant, self.values[constant as usize]);
        }
        else {
            println!("Expected constant literal at offset {} but found unknown code", constant_offset);
        }
        constant_offset + 1
    }
}

#[derive(Debug)]
struct Value(f64);

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}