use chunk::{Chunk, Instruction};
use interpreter::Interpreter;

mod interpreter;
mod chunk;

fn main() {
    let mut chunk = Chunk::new();
    chunk.write_instruction(Instruction::Constant(1.2), 125);
    chunk.write_instruction(Instruction::Constant(35.0), 125);
    chunk.write_instruction(Instruction::Return, 128);
    chunk.disassemble("Test chunk");

    let mut interpreter = Interpreter::new(chunk);
    match interpreter.run() {
        Ok(_) => {},
        Err(e) => println!("{}", e)
    }
}
