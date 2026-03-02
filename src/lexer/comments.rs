// src/lexer/comments.rs
use crate::lexer::Lexer;

impl Lexer {
    pub fn skip_comment(&mut self) {
        // Skip until end of line
        while self.ch != '\n' && self.ch != '\0' {
            self.read_char();
        }
        // Skip the newline as well
        if self.ch == '\n' {
            self.line += 1;
            self.column = 0;
            self.read_char();
        }
    }
}
