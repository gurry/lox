use std::{path::{PathBuf, Path}, fs::read_to_string, io::{self, Write, BufRead}};

use anyhow::{Context, Result, bail};
use compiler::{Compiler, CompileErrorCollection};
use disassembler::Disassembler;
use structopt::StructOpt;
use vm::Vm;

mod vm;
mod chunk;
mod disassembler;
mod instruction;
mod stack;
mod scanner;
mod compiler;


#[derive(Debug, StructOpt)]
#[structopt()]
struct Options {
    /// Output file, stdout if not present
    #[structopt(parse(from_os_str))]
    source_file_path: Option<PathBuf>,

    #[structopt(short, long)]
    trace: bool,

    #[structopt(short="d", long="dasm")]
    disassemble: bool
}

fn main() -> Result<()> {
    let Options { source_file_path, trace , disassemble} = Options::from_args();
    match source_file_path {
        Some(path) => run_file(&path, trace, disassemble),
        None => run_prompt(trace, disassemble)
    }
}

fn run_file(source_file_path: &Path, trace: bool, disassemble: bool) -> Result<()> {
    let source = read_to_string(source_file_path).context("Failed to read source file")?;
    run(source, trace, disassemble);
    Ok(())
}

fn run_prompt(trace: bool, disassemble: bool) -> Result<()> {
    loop {
        print!("> ");
        io::stdout().flush().context("Failed to flush stdout")?;
        let mut line = String::new();
        let stdin = io::stdin();
        stdin.lock().read_line(&mut line).context("stdin failed")?;
        run(line, trace, disassemble);
        println!("");
    }
}

fn run(source: String, trace: bool, disassemble: bool) {
    let compiler = Compiler::new(source);
    let mut chunk = match compiler.compile() {
        Ok(c) => c,
        Err(e) => {
           match &e.downcast_ref::<CompileErrorCollection>() {
                Some(ce) => {
                    for e in &ce.errors {
                        println!("{}", e);
                    }
                },
                None => {
                    println!("Error occured: {}", e);
                }
            };

            return;
        }
    };

    if disassemble {
        let mut disassembler = Disassembler::new();
        match disassembler.disassemble(&chunk, "Chunk") {
            Ok(_) => println!(),
            Err(e) => {
                println!("Disassembly failed: {}", e);
                return;
            }
        }
    } 

    let mut vm = Vm::new(trace);
    match vm.run(&mut chunk) {
        Err(e) => println!("Code execution failed: {}", e),
        _ => {}
    };
}
