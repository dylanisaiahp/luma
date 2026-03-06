// src/cli/mod.rs
use crate::ast::Stmt;
use crate::debug::DebugConfig;
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
    #[command(trailing_var_arg = true)]
    Run {
        file: String,
        #[arg(long, help = "Show execution time")]
        time: bool,
        #[arg(
            long,
            num_args = 1..,
            help = "Debug components: lexer, parser, interpreter, all (append :verbose for more detail)"
        )]
        debug: Vec<String>,
        /// Extra arguments passed to the Luma program via input()
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Check a Luma file for errors
    Check { file: String },
}

pub fn execute_command(command: Commands) -> anyhow::Result<()> {
    match command {
        Commands::New { name } => create_project(&name),
        Commands::Run {
            file,
            time,
            debug,
            args: _,
        } => {
            let flags: Vec<&str> = debug.iter().map(|s| s.as_str()).collect();
            let config = DebugConfig::from_flags(&flags);
            run_file(&file, time, config)
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

fn run_file(file: &str, show_time: bool, debug: DebugConfig) -> anyhow::Result<()> {
    let start = Instant::now();

    let source = fs::read_to_string(file)?;
    let mut collector = ErrorCollector::new(&source, file);

    // Lexing
    let mut lexer = crate::lexer::Lexer::new(&source);
    let (tokens, lex_errors) = lexer.tokenize();

    if debug.lexer {
        crate::debug::lexer::print_lexer_debug(&tokens, &lex_errors, debug.verbose);
    }

    for error in lex_errors {
        collector.add_lexer_error(error);
    }

    // Parsing
    let mut parser = crate::parser::Parser::new(tokens);
    let statements = parser.parse_program();
    let parse_errors = parser.take_errors();
    let parse_error_count = parse_errors.len();

    if debug.parser {
        crate::debug::parser::print_parser_debug(&statements, parse_error_count, debug.verbose);
    }

    for error in parse_errors {
        collector.add_parse_error(error);
    }

    // Only interpret if parsing succeeded
    if !collector.has_errors() {
        let ast = Stmt::Program(statements);
        let mut interpreter = crate::interpreter::Interpreter::new();
        interpreter.debug_mode = debug.interpreter || debug.lexer || debug.parser;

        match interpreter.interpret(&ast, &source, file) {
            Ok(()) => {
                if debug.interpreter {
                    interpreter.debug.print_debug(debug.verbose);
                }
                // Flush buffered output after debug
                for line in &interpreter.output_buffer {
                    println!("{}", line);
                }
                for warning in interpreter.take_warnings() {
                    collector.add_warning(warning);
                }
            }
            Err(e) => {
                if debug.interpreter {
                    interpreter.debug.print_debug(debug.verbose);
                }
                for line in &interpreter.output_buffer {
                    println!("{}", line);
                }
                collector.add_runtime_error(e.message, "".to_string(), e.line, e.column);
            }
        }
    }

    // Print errors after debug output with spacing
    if collector.has_errors() {
        println!();
    }
    collector.print_all();

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
