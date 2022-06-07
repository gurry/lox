use std::{path::{PathBuf, Path}, fs::read_to_string, io::{self, Write, BufRead}};

use anyhow::{Context, Result, bail};
use compiler::{Compiler, CompileErrorCollection};
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
    trace: bool
}

fn main() -> Result<()> {
    let Options { source_file_path, trace } = Options::from_args();
    match source_file_path {
        Some(path) => run_file(&path, trace),
        None => run_prompt(trace)
    }
}

fn run_file(source_file_path: &Path, trace: bool) -> Result<()> {
    let source = read_to_string(source_file_path).context("Failed to read source file")?;
    run(source, trace)
}

fn run_prompt(trace: bool) -> Result<()> {
    loop {
        print!("> ");
        io::stdout().flush().context("Failed to flush stdout")?;
        let mut line = String::new();
        let stdin = io::stdin();
        stdin.lock().read_line(&mut line).context("stdin failed")?;
        run(line, trace)?;
        println!("");
    }
}

fn run(source: String, trace: bool) -> Result<()> {
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
                None => println!("Error: {}", e),
            };

            bail!("Compilation failed");
        }
    };

    let mut vm = Vm::new(trace);
    vm.run(&mut chunk).context("VM failed")?;

    Ok(())
}
