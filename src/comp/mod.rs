// src/comp/mod.rs
mod codegen;

use crate::ast::Stmt;
use std::fs;
use std::path::Path;
use std::process::Command;

pub use codegen::Codegen;

/// The Luma runtime, embedded at compile time.
/// Written to disk alongside generated code during luma build.
const RUNTIME_SRC: &str = include_str!("runtime.rs");

pub struct CompileOptions {
    pub output_name: String,
    pub output_dir: String,
    pub source_file: Option<String>,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            output_name: "program".to_string(),
            output_dir: "builds".to_string(),
            source_file: None,
        }
    }
}

pub fn compile(stmts: Vec<Stmt>, options: CompileOptions) -> anyhow::Result<()> {
    fs::create_dir_all(&options.output_dir)?;
    let build_dir = Path::new(&options.output_dir);

    // Write the embedded runtime to the build dir
    let runtime_path = build_dir.join("luma_runtime.rs");
    fs::write(&runtime_path, RUNTIME_SRC)?;

    // Generate Rust source from AST
    let codegen = match &options.source_file {
        Some(file) => Codegen::new().with_file(file),
        None => Codegen::new(),
    };
    let generated = codegen.generate(&stmts);
    let source_path = build_dir.join(format!("{}.rs", options.output_name));
    fs::write(&source_path, &generated)?;

    // Compile with rustc from the build dir so it can find luma_runtime
    println!("{:<36} compiling...", options.output_name);

    let status = Command::new("rustc")
        .arg(format!("{}.rs", options.output_name))
        .arg("--edition")
        .arg("2021")
        .arg("-o")
        .arg(&options.output_name)
        .current_dir(build_dir)
        .status()?;

    if !status.success() {
        anyhow::bail!(
            "rustc failed — check {}/{}.rs for details.",
            options.output_dir,
            options.output_name
        );
    }

    println!("built {}/{} ✓", options.output_dir, options.output_name);
    Ok(())
}
