// src/debug/parser.rs
use crate::ast::Stmt;
use crate::debug::format::{self, level_tag, parser_tag};

pub fn print_parser_debug(statements: &[Stmt], error_count: usize, verbose: bool) {
    let tag = parser_tag();
    let level = level_tag(verbose);

    let functions: Vec<&Stmt> = statements
        .iter()
        .filter(|s| matches!(s, Stmt::Function { .. }))
        .collect();

    println!("{}", level);
    println!("{}", format::top(&tag, "Starting parse"));

    for stmt in &functions {
        if let Stmt::Function {
            name,
            return_type,
            params,
            body,
        } = stmt
        {
            println!(
                "{}",
                format::mid(&tag, &format!("function: {} ({})", name, return_type))
            );

            if verbose {
                for (i, param) in params.iter().enumerate() {
                    let is_last = i == params.len() - 1 && body.is_empty();
                    if is_last {
                        println!(
                            "{}",
                            format::mid_indent_last(
                                &tag,
                                &format!("param: {} ({})", param.name, param.type_name)
                            )
                        );
                    } else {
                        println!(
                            "{}",
                            format::mid_indent(
                                &tag,
                                &format!("param: {} ({})", param.name, param.type_name)
                            )
                        );
                    }
                }
                println!(
                    "{}",
                    format::mid_indent_last(&tag, &format!("{} statements", body.len()))
                );
            }
        }
    }

    println!(
        "{}",
        format::bot(
            &tag,
            &format!("{} functions  •  {} errors", functions.len(), error_count)
        )
    );
    println!();
}
