use anyhow::Context;
use chunk::Chunk;
use disassembler::Disassembler;
use instruction::{InstructionWriter, OpCode};
use vm::Vm;

mod vm;
mod chunk;
mod disassembler;
mod instruction;
mod stack;

fn main() -> anyhow::Result<()> {
    let mut chunk = Chunk::new();
    let mut writer = InstructionWriter::new(&mut chunk);
    writer.write_const(1.2, 125);
    writer.write_const(35.0, 125);
    writer.write_op_code(OpCode::Negate, 128);
    writer.write_op_code(OpCode::Return, 128);

    // let mut disassembler = Disassembler::new();
    // disassembler.disassemble(&chunk,"Test chunk")
    //     .with_context(|| "Disassembler failed")?;

    let mut interpreter = Vm::new_with_tracing(chunk);
    interpreter.run().with_context(|| "Interpreter failed")
}
