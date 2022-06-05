use anyhow::Context;
use chunk::Chunk;
use instruction::{InstructionWriter, OpCode};
use vm::Vm;

mod vm;
mod chunk;
mod disassembler;
mod instruction;
mod stack;
mod scanner;
mod token;

fn main() -> anyhow::Result<()> {
    let mut chunk = Chunk::new();
    let mut writer = InstructionWriter::new(&mut chunk);
    writer.write_const(1.2, 125);
    writer.write_const(35.0, 125);
    writer.write_op_code(OpCode::Negate, 127);
    writer.write_op_code(OpCode::Add, 128);
    writer.write_const(5.0, 129);
    writer.write_op_code(OpCode::Multiply, 130);
    writer.write_op_code(OpCode::Return, 131);

    // let mut disassembler = Disassembler::new();
    // disassembler.disassemble(&chunk,"Test chunk")
    //     .with_context(|| "Disassembler failed")?;

    let mut interpreter = Vm::new_with_tracing();
    interpreter.run(&mut chunk).with_context(|| "VM failed")
}
