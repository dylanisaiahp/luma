// src/cli/mod.rs
use crate::ast::Stmt;
use crate::debug::DebugLevel;
use crate::error::ErrorCollector;

use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "luma", version, about = "A small, clean programming language")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new Luma project
    New { name: String },

    /// Run a Luma file
    Run {
        file: String,
        #[arg(long, help = "Show execution time")]
        time: bool,
        #[arg(long, help = "Debug level (basic or verbose)")]
        debug: Option<String>,
    },

    /// Check a Luma file for errors
    Check { file: String },
}

pub fn execute_command(command: Commands) -> anyhow::Result<()> {
    match command {
        Commands::New { name } => create_project(&name),
        Commands::Run { file, time, debug } => {
            if let Some(level) = debug {
                match level.as_str() {
                    "basic" => crate::debug::set_level(crate::debug::DebugLevel::Basic),
                    "verbose" => crate::debug::set_level(crate::debug::DebugLevel::Verbose),
                    "trace" => crate::debug::set_level(crate::debug::DebugLevel::Trace),
                    _ => eprintln!("Unknown debug level: {}", level),
                }
            }
            run_file(&file, time)
        }
        Commands::Check { file } => check_file(&file),
    }
}

fn create_project(name: &str) -> anyhow::Result<()> {
    let path = Path::new(name);

    if path.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    fs::create_dir(path)?;
    fs::create_dir(path.join("lm"))?;

    let main_content = r#"void main() {
    print("Hello, Luma!");
}
"#;
    fs::write(path.join("lm/main.lm"), main_content)?;

    let readme = format!("# {}\n\nA Luma project.\n", name);
    fs::write(path.join("README.md"), readme)?;

    println!("[✓] Created new Luma project: {}", name);
    println!("    cd {} to get started", name);
    println!("    lm/main.lm is your entry point");

    Ok(())
}

fn run_file(file: &str, show_time: bool) -> anyhow::Result<()> {
    crate::debug!(DebugLevel::Basic, "Running file: {}", file);
    let start = Instant::now();

    let source = fs::read_to_string(file)?;
    let mut collector = ErrorCollector::new(&source, file);

    // Lexing
    let mut lexer = crate::lexer::Lexer::new(&source);
    let (tokens, lex_errors) = lexer.tokenize();

    // Add any lexer errors to the collector
    for error in lex_errors {
        collector.add_lexer_error(error);
    }

    // Parsing
    let mut parser = crate::parser::Parser::new(tokens);
    let statements = parser.parse_program();
    for error in parser.take_errors() {
        collector.add_parse_error(error);
    }

    // Only interpret if parsing succeeded
    if !collector.has_errors() {
        let ast = Stmt::Program(statements);
        let mut interpreter = crate::interpreter::Interpreter::new();

        match interpreter.interpret(&ast, &source, file) {
            Ok(()) => {
                for warning in interpreter.take_warnings() {
                    collector.add_warning(warning);
                }
            }
            Err(e) => {
                collector.add_runtime_error(e.message, "".to_string(), e.line, e.column);
            }
        }
    }

    // Now print everything together
    collector.print_all();

    // Exit once, at the very end
    if collector.has_errors() {
        std::process::exit(1);
    }

    if show_time {
        let duration = start.elapsed();
        println!("\n⚡ Completed in {:?}", duration);
    }
    Ok(())
}

fn check_file(file: &str) -> anyhow::Result<()> {
    let source = fs::read_to_string(file)?;
    let mut collector = ErrorCollector::new(&source, file);

    let mut lexer = crate::lexer::Lexer::new(&source);
    let (tokens, lex_errors) = lexer.tokenize();

    // Add any lexer errors to the collector
    for error in lex_errors {
        collector.add_lexer_error(error);
    }

    let mut parser = crate::parser::Parser::new(tokens);
    let _statements = parser.parse_program();
    for error in parser.take_errors() {
        collector.add_parse_error(error);
    }

    collector.print_all();

    if collector.has_errors() {
        anyhow::bail!("Compilation failed");
    }

    println!("[✓] Everything looks good!");
    Ok(())
}
