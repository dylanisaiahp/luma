// src/debug/lexer.rs
use crate::debug::format::{self, level_tag, lexer_tag};
use crate::lexer::{Token, TokenKind};

pub fn print_lexer_debug(tokens: &[Token], errors: &[crate::lexer::LexerError], verbose: bool) {
    let tag = lexer_tag();
    let level = level_tag(verbose);
    let line_count = tokens.iter().map(|t| t.line).max().unwrap_or(0);
    let token_count = tokens.iter().filter(|t| t.kind != TokenKind::Eof).count();

    println!("{}", level);
    println!("{}", format::top(&tag, "Tokenizing"));
    println!(
        "{}",
        format::mid(
            &tag,
            &format!(
                "{} tokens  •  {} lines  •  {} errors",
                token_count,
                line_count,
                errors.len()
            )
        )
    );

    if verbose {
        for token in tokens.iter().filter(|t| t.kind != TokenKind::Eof) {
            println!(
                "{}",
                format::mid(
                    &tag,
                    &format!("{:<20} line {}", format!("{}", token.kind), token.line)
                )
            );
        }
    }

    println!("{}", format::bot(&tag, "Done"));
    println!();
}
