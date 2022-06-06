use std::{path::{PathBuf, Path}, fs::read_to_string, io::{self, Write, BufRead}};

use anyhow::{Context, Result};
use scanner::Scanner;
use structopt::StructOpt;

mod vm;
mod chunk;
mod disassembler;
mod instruction;
mod stack;
mod scanner;


#[derive(Debug, StructOpt)]
#[structopt()]
struct Options {
    /// Output file, stdout if not present
    #[structopt(parse(from_os_str))]
    source_file_path: Option<PathBuf>
}

fn main() -> Result<()> {
    match Options::from_args() {
        Options { source_file_path: Some(source_file_path) } => run_file(&source_file_path),
        _ => run_prompt()
    }
}

fn run_file(source_file_path: &Path) -> Result<()> {
    let source = read_to_string(source_file_path).context("Failed to read source file")?;
    run(source)
}

fn run_prompt() -> Result<()> {
    loop {
        print!("> ");
        io::stdout().flush().context("Failed to flush stdout")?;
        let mut line = String::new();
        let stdin = io::stdin();
        stdin.lock().read_line(&mut line).context("stdin failed")?;
        run(line)?;
        println!("");
    }
}

fn run(source: String) -> Result<()> {
    let mut scanner = Scanner::new(source);
    loop {
        let token = scanner.scan_next()
            .context("Scanner failed")?; 

        if token.token_type == scanner::TokenType::Eof {
            break;
        }
        
        println!("{:?}", token);
    }

    // TODO: compile tokens into chunk here

    // let mut vm = Vm::new_with_tracing();
    // vm.run(&mut chunk).with_context(|| "VM failed")


    Ok(())
}
