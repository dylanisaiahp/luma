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
    /// Create a new Luma project
    New { name: String },

    /// Create a new .lm file or directory
    Create { path: String },

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

fn print_step(label: &str) {
    println!("{:<36} done.", label);
}

pub fn execute_command(command: Commands) -> anyhow::Result<()> {
    match command {
        Commands::New { name } => create_project(&name),
        Commands::Create { path } => create_path(&path),
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

fn resolve_use(module_name: &str, importing_file: &str) -> Option<PathBuf> {
    if let Some(path) = find_module_declaration(Path::new("source"), module_name) {
        return Some(path);
    }

    if let Some(parent) = Path::new(importing_file).parent()
        && let Some(path) = find_module_declaration(parent, module_name)
    {
        return Some(path);
    }

    None
}

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

    let mut expanded: Vec<Stmt> = Vec::new();
    for stmt in statements {
        match &stmt {
            Stmt::Use { module, items } => match resolve_use(module, file_path) {
                Some(mod_path) => {
                    let mod_path_str = mod_path.to_string_lossy().to_string();
                    let mut mod_stmts = load_with_uses(&mod_path_str, visited, collector, debug);

                    if let Some(selected) = items {
                        mod_stmts.retain(|s| match s {
                            Stmt::Function { name, .. } => selected.contains(name),
                            Stmt::StructDeclaration { name, .. } => selected.contains(name),
                            _ => true,
                        });
                    }

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
            },
            _ => expanded.push(stmt),
        }
    }

    expanded
}

fn create_project(name: &str) -> anyhow::Result<()> {
    let path = Path::new(name);

    if path.exists() {
        anyhow::bail!("[!] Tried to create {}/ but it already exists", name);
    }

    print_step(&format!("creating {}/", name));
    fs::create_dir(path)?;

    print_step(&format!("creating {}/source/", name));
    fs::create_dir(path.join("source"))?;

    print_step(&format!("creating {}/source/main.lm", name));
    let main_content = "void main() {\n    print(\"Hello, Luma!\");\n}\n";
    fs::write(path.join("source/main.lm"), main_content)?;

    print_step(&format!("creating {}/luma.toml", name));
    let toml_content = format!(
        "[project]\nname = \"{}\"\nversion = \"0.1.0\"\ndescription = \"\"\nentry = \"source/main.lm\"\n",
        name
    );
    fs::write(path.join("luma.toml"), toml_content)?;

    print_step(&format!("creating {}/README.md", name));
    let readme = format!("# {}\n\nA Luma project.\n", name);
    fs::write(path.join("README.md"), readme)?;

    print_step(&format!("creating {}/.gitignore", name));
    let gitignore = "# Luma build output\nbuilds/\n\n# OS\n.DS_Store\nThumbs.db\n";
    fs::write(path.join(".gitignore"), gitignore)?;

    let git_label = "initializing git";
    let git_result = std::process::Command::new("git")
        .arg("init")
        .current_dir(path)
        .output();

    match git_result {
        Ok(output) if output.status.success() => {
            print_step(git_label);
            println!();
            println!("{} created successfully ✓", name);
            println!("git initialized ✓");
            println!();
            println!("next -> cd {} && luma run", name);
        }
        _ => {
            println!("{:<36} failed.", git_label);
            println!();
            println!("{} created successfully ✓", name);
            println!("[!] git init failed — is git installed?");
            println!();
            println!("next -> cd {} && luma run", name);
        }
    }

    Ok(())
}

/// Create a .lm file or a directory.
/// Trailing slash → directory only.
/// No trailing slash → .lm file with module declaration.
fn create_path(raw_path: &str) -> anyhow::Result<()> {
    // Directory: trailing slash
    if raw_path.ends_with('/') {
        let dir_path = Path::new(raw_path);
        if dir_path.exists() {
            anyhow::bail!("[!] Tried to create {} but it already exists", raw_path);
        }
        print_step(&format!("creating {}", raw_path));
        fs::create_dir_all(dir_path)?;
        println!();
        println!("{} created successfully ✓", raw_path);
        return Ok(());
    }

    // File: strip .lm if provided, then add it back
    let stem = raw_path.trim_end_matches(".lm");
    let file_path = format!("{}.lm", stem);
    let path = Path::new(&file_path);

    if path.exists() {
        anyhow::bail!("[!] Tried to create {} but it already exists", file_path);
    }

    // Create parent directories if needed
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        print_step(&format!("creating {}/", parent.display()));
        fs::create_dir_all(parent)?;
    }

    // Module name is the filename stem only, not the full path
    let module_name = Path::new(stem)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(stem);

    print_step(&format!("creating {}", file_path));
    let content = format!("module {};\n", module_name);
    fs::write(path, &content)?;

    println!();
    println!("{} created successfully ✓", file_path);
    println!();
    println!("next -> use {};", module_name);

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
