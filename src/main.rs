use anyhow::Context;
use chunk::Chunk;
use disassembler::Disassembler;
use instruction::{InstructionWriter, Instruction};
use interpreter::Interpreter;

mod interpreter;
mod chunk;
mod disassembler;
mod instruction;

fn main() -> anyhow::Result<()> {
    let mut chunk = Chunk::new();
    let mut writer = InstructionWriter::new(&mut chunk);
    writer.write(Instruction::Constant(1.2), 125);
    writer.write(Instruction::Constant(35.0), 125);
    writer.write(Instruction::Return, 128);

    Disassembler::disassemble(&chunk,"Test chunk")
        .with_context(|| "Disassembler failed")?;

    let mut interpreter = Interpreter::new(chunk);
    interpreter.run().with_context(|| "Interpreter failed")
}
