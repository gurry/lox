use interpreter::{Instruction, Chunk};

mod interpreter;

fn main() {
    let mut chunk = Chunk::new();
    chunk.write_instruction(Instruction::Constant(1.2), 125);
    chunk.write_instruction(Instruction::Constant(35.0), 125);
    chunk.write_instruction(Instruction::Return, 128);
    chunk.disassemble("Test chunk");
}
