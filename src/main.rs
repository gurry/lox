use anyhow::Context;
use chunk::Chunk;
use disassembler::Disassembler;
use instruction::InstructionWriter;
use interpreter::Interpreter;

mod interpreter;
mod chunk;
mod disassembler;
mod instruction;
mod stack;

fn main() -> anyhow::Result<()> {
    let mut chunk = Chunk::new();
    let mut writer = InstructionWriter::new(&mut chunk);
    writer.write_const(1.2, 125);
    writer.write_const(35.0, 125);
    writer.write_return(128);

    // let mut disassembler = Disassembler::new();
    // disassembler.disassemble(&chunk,"Test chunk")
    //     .with_context(|| "Disassembler failed")?;

    let mut interpreter = Interpreter::new_with_tracing(chunk);
    interpreter.run().with_context(|| "Interpreter failed")
}
