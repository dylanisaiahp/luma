// src/debug/interpreter.rs
use crate::debug::format::{self, interpreter_tag, level_tag};

pub struct InterpreterDebug {
    pub events: Vec<DebugEvent>,
}

pub enum DebugEvent {
    Print { value: String },
}

impl InterpreterDebug {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn log_print(&mut self, value: &str) {
        self.events.push(DebugEvent::Print {
            value: value.to_string(),
        });
    }

    pub fn print_debug(&self, verbose: bool, filename: &str) {
        let tag = interpreter_tag();
        let level = level_tag(verbose, Some(filename));

        println!("{}", level);
        println!("{}", format::top(&tag, "Executing main()"));

        let len = self.events.len();
        for (i, event) in self.events.iter().enumerate() {
            let is_last = i == len - 1;
            let line = match event {
                DebugEvent::Print { value } => {
                    format!("printed: {}", value)
                }
            };
            if is_last {
                println!("{}", format::bot(&tag, &line));
            } else {
                println!("{}", format::mid(&tag, &line));
            }
        }

        if self.events.is_empty() {
            println!("{}", format::bot(&tag, "Done"));
        }
        println!();
    }
}

impl Default for InterpreterDebug {
    fn default() -> Self {
        Self::new()
    }
}
