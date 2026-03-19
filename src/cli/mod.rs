// src/cli/mod.rs
use crate::ast::Stmt;
use crate::debug::DebugConfig;
use crate::error::ErrorCollector;

use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Parser)]
#[command(name = "luma", version, about = "A small, clean programming language")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new Luma project or file
    New {
        name: String,
        #[arg(long, help = "Create a single .lm file instead of a project")]
        file: bool,
    },

    /// Run a Luma file or project
    #[command(trailing_var_arg = true)]
    Run {
        /// File to run (optional — reads luma.toml entry if omitted)
        file: Option<String>,
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

/// Parsed luma.toml
#[derive(Debug)]
struct LumaToml {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    version: String,
    #[allow(dead_code)]
    description: String,
    entry: Option<String>,
}

impl LumaToml {
    fn parse(content: &str) -> Self {
        let mut name = String::new();
        let mut version = String::new();
        let mut description = String::new();
        let mut entry = None;

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                continue;
            }
            if let Some(val) = parse_toml_string(line, "name") {
                name = val;
            } else if let Some(val) = parse_toml_string(line, "version") {
                version = val;
            } else if let Some(val) = parse_toml_string(line, "description") {
                description = val;
            } else if let Some(val) = parse_toml_string(line, "entry") {
                entry = Some(val);
            }
        }

        LumaToml {
            name,
            version,
            description,
            entry,
        }
    }
}

fn parse_toml_string(line: &str, key: &str) -> Option<String> {
    let prefix = format!("{} =", key);
    if !line.starts_with(&prefix) {
        return None;
    }
    let rest = line[prefix.len()..].trim();
    let rest = rest.strip_prefix('"')?.strip_suffix('"')?;
    Some(rest.to_string())
}

pub fn execute_command(command: Commands) -> anyhow::Result<()> {
    match command {
        Commands::New { name, file } => {
            if file {
                create_file_lm(&name)
            } else {
                create_project(&name)
            }
        }
        Commands::Run {
            file,
            time,
            debug,
            args: _,
        } => {
            let flags: Vec<&str> = debug.iter().map(|s| s.as_str()).collect();
            let config = DebugConfig::from_flags(&flags);

            let target = match file {
                Some(f) => f,
                None => resolve_entry_from_toml()?,
            };

            run_file(&target, time, config)
        }
        Commands::Check { file } => check_file(&file),
    }
}

/// Read luma.toml in the current directory and resolve the entry file path.
fn resolve_entry_from_toml() -> anyhow::Result<String> {
    let toml_path = Path::new("luma.toml");
    if !toml_path.exists() {
        anyhow::bail!(
            "No file specified and no luma.toml found.\n\
             Run 'luma run <file>' or create a luma.toml with an entry point."
        );
    }

    let content = fs::read_to_string(toml_path)?;
    let config = LumaToml::parse(&content);

    match config.entry {
        Some(entry) => {
            if Path::new(&entry).exists() {
                return Ok(entry);
            }
            let filename = Path::new(&entry)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&entry)
                .to_string();
            if let Some(found) = find_in_source(&filename) {
                return Ok(found.to_string_lossy().to_string());
            }
            anyhow::bail!(
                "Entry point '{}' not found.\n\
                 Check your luma.toml or run 'luma run <file>' directly.",
                entry
            )
        }
        None => anyhow::bail!(
            "luma.toml has no entry point defined.\n\
             Add 'entry = \"source/main.lm\"' or run 'luma run <file>' directly."
        ),
    }
}

fn find_in_source(filename: &str) -> Option<PathBuf> {
    search_dir(Path::new("source"), filename)
}

fn search_dir(dir: &Path, filename: &str) -> Option<PathBuf> {
    let entries = fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = search_dir(&path, filename) {
                return Some(found);
            }
        } else if path.file_name().and_then(|n| n.to_str()) == Some(filename) {
            return Some(path);
        }
    }
    None
}

/// Resolve a `use` module name to a file path.
///
/// Rules (new system):
///   - Scan source/ recursively for a file containing `module <name>;`
///   - The module declaration file CAN have the same name as the module (e.g. parser.lm with module parser;)
///   - No filename fallback — every importable file must declare its module name
///   - Check same directory as the importing file as a final fallback
fn resolve_use(module_name: &str, importing_file: &str) -> Option<PathBuf> {
    // 1. Scan source/ for a file declaring `module <name>;`
    if let Some(path) = find_module_declaration(Path::new("source"), module_name) {
        return Some(path);
    }

    // 2. Same directory as importing file
    if let Some(parent) = Path::new(importing_file).parent()
        && let Some(path) = find_module_declaration(parent, module_name)
    {
        return Some(path);
    }

    None
}

/// Recursively search dir for a .lm file that contains `module <name>;`
fn find_module_declaration(dir: &Path, module_name: &str) -> Option<PathBuf> {
    let entries = fs::read_dir(dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_module_declaration(&path, module_name) {
                return Some(found);
            }
        } else if path.extension().and_then(|e| e.to_str()) == Some("lm")
            && let Ok(content) = fs::read_to_string(&path)
            && content
                .lines()
                .any(|l| l.trim() == format!("module {};", module_name))
        {
            return Some(path);
        }
    }
    None
}

/// Load all statements from a use chain starting at `file_path`.
/// Handles `use` statements recursively, avoiding cycles.
fn load_with_uses(
    file_path: &str,
    visited: &mut std::collections::HashSet<String>,
    collector: &mut ErrorCollector,
    debug: &DebugConfig,
) -> Vec<Stmt> {
    let canonical = fs::canonicalize(file_path)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| file_path.to_string());

    if visited.contains(&canonical) {
        return Vec::new();
    }
    visited.insert(canonical);

    let source = match fs::read_to_string(file_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[!] Could not read '{}': {}", file_path, e);
            return Vec::new();
        }
    };

    let mut lexer = crate::lexer::Lexer::new(&source);
    let (tokens, lex_errors) = lexer.tokenize();

    if debug.lexer {
        crate::debug::lexer::print_lexer_debug(&tokens, &lex_errors, debug.verbose);
    }

    for error in lex_errors {
        collector.add_lexer_error(error);
    }

    let mut parser = crate::parser::Parser::new(tokens);
    let statements = parser.parse_program();
    let parse_errors = parser.take_errors();

    if debug.parser {
        crate::debug::parser::print_parser_debug(&statements, parse_errors.len(), debug.verbose);
    }

    for error in parse_errors {
        collector.add_parse_error(error);
    }

    // Expand use statements
    let mut expanded: Vec<Stmt> = Vec::new();
    for stmt in statements {
        match &stmt {
            Stmt::Use { module, items } => {
                match resolve_use(module, file_path) {
                    Some(mod_path) => {
                        let mod_path_str = mod_path.to_string_lossy().to_string();
                        let mut mod_stmts =
                            load_with_uses(&mod_path_str, visited, collector, debug);

                        // Selective import: use http.(client, request)
                        if let Some(selected) = items {
                            mod_stmts.retain(|s| match s {
                                Stmt::Function { name, .. } => selected.contains(name),
                                Stmt::StructDeclaration { name, .. } => selected.contains(name),
                                _ => true,
                            });
                        }

                        // Skip ModuleDeclaration stmts — metadata only
                        for s in mod_stmts {
                            if !matches!(s, Stmt::ModuleDeclaration { .. }) {
                                expanded.push(s);
                            }
                        }
                    }
                    None => {
                        eprintln!(
                            "[!] Could not resolve module '{}' (imported in '{}')\n    \
                             Make sure the file contains: module {};",
                            module, file_path, module
                        );
                    }
                }
            }
            _ => expanded.push(stmt),
        }
    }

    expanded
}

fn create_project(name: &str) -> anyhow::Result<()> {
    let path = Path::new(name);

    if path.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    fs::create_dir(path)?;
    fs::create_dir(path.join("source"))?;

    let main_content = "void main() {\n    print(\"Hello, Luma!\");\n}\n";
    fs::write(path.join("source/main.lm"), main_content)?;

    let toml_content = format!(
        "[project]\nname = \"{}\"\nversion = \"0.1.0\"\ndescription = \"\"\nentry = \"source/main.lm\"\n",
        name
    );
    fs::write(path.join("luma.toml"), toml_content)?;

    let readme = format!("# {}\n\nA Luma project.\n", name);
    fs::write(path.join("README.md"), readme)?;

    let gitignore = "# Luma build output\n.luma/\n\n# OS\n.DS_Store\nThumbs.db\n";
    fs::write(path.join(".gitignore"), gitignore)?;

    let git_result = std::process::Command::new("git")
        .arg("init")
        .current_dir(path)
        .output();

    match git_result {
        Ok(output) if output.status.success() => {
            println!("[✓] Created new Luma project: {}", name);
            println!("    cd {}", name);
            println!("    luma run");
            println!("    git initialized");
        }
        _ => {
            println!("[✓] Created new Luma project: {}", name);
            println!("    cd {}", name);
            println!("    luma run");
            println!("    [!] git init failed — is git installed?");
        }
    }

    Ok(())
}

/// Create a single .lm file with a module declaration at the top.
fn create_file_lm(name: &str) -> anyhow::Result<()> {
    // Strip .lm extension if provided, derive module name from filename stem
    let stem = name.trim_end_matches(".lm");
    let filename = format!("{}.lm", stem);
    let path = Path::new(&filename);

    if path.exists() {
        anyhow::bail!("File '{}' already exists", filename);
    }

    let content = format!("module {};\n", stem);
    fs::write(path, &content)?;

    println!("[✓] Created {}", filename);
    Ok(())
}

fn run_file(file: &str, show_time: bool, debug: DebugConfig) -> anyhow::Result<()> {
    let start = Instant::now();

    let source = fs::read_to_string(file)?;
    let mut collector = ErrorCollector::new(&source, file);

    let mut visited = std::collections::HashSet::new();
    let statements = load_with_uses(file, &mut visited, &mut collector, &debug);

    if collector.has_errors() {
        println!();
        collector.print_all();
        std::process::exit(1);
    }

    let ast = Stmt::Program(statements);
    let mut interpreter = crate::interpreter::Interpreter::new();
    interpreter.debug_mode = debug.interpreter || debug.lexer || debug.parser;

    match interpreter.interpret(&ast, &source, file) {
        Ok(()) => {
            if debug.interpreter {
                interpreter.debug.print_debug(debug.verbose);
            }
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
