// src/main.rs
mod ast;
mod cli;
mod comp;
mod debug;
mod error;
mod interpreter;
mod lexer;
mod parser;
mod syntax;

use clap::Parser;

fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();
    cli::execute_command(cli.command)
}
