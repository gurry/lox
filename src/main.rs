use std::fmt::Display;

fn main() {
    let mut chunk = Chunk::new();
    chunk.write_instruction(Instruction::Constant(1.2), 125);
    chunk.write_instruction(Instruction::Constant(35.0), 125);
    chunk.write_instruction(Instruction::Return, 128);
    chunk.disassemble("Test chunk");
}

enum Instruction {
    Constant(f64),
    Return
}

#[derive(Debug, Clone)]
#[repr(u8)]
enum CodeByte {
    Literal(u8),
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
    line_numbers: Vec<i32>,
    values: Vec<Value>
}

impl Chunk {
    fn new() -> Self { 
        Self { code: Vec::new(), line_numbers: Vec::new(), values: Vec::new() }
    }

    fn write_instruction(&mut self, instruction: Instruction, line_number: i32)  {
        match instruction {
            Instruction::Constant(const_value) => {
                let const_index = self.add_constant(Value(const_value));
                self.write_code(CodeByte::OpCode(OpCode::Constant), line_number);
                self.write_code(CodeByte::Literal(const_index), line_number);
            },
            Instruction::Return => self.write_code(CodeByte::OpCode(OpCode::Return), line_number),
        }
        
    }

    fn write_code(&mut self, code_byte: CodeByte, line_number: i32)  {
        self.code.push(code_byte);
        self.line_numbers.push(line_number);
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

        let prev_line_number = if offset > 0 { Some(self.line_numbers[offset - 1]) } else { None };
        let line_number = self.line_numbers[offset];
        let same_line_no_as_previous = prev_line_number.is_some() && line_number == prev_line_number.unwrap();
        if same_line_no_as_previous {
            print!("   | ");
        } else {
            print!("{:4} ", line_number);
        }

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
        if let CodeByte::Literal(constant) = self.code[constant_offset] {
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