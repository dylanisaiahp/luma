// src/debug.rs
#![allow(dead_code)]
use std::sync::atomic::{AtomicUsize, Ordering};

static DEBUG_LEVEL: AtomicUsize = AtomicUsize::new(0);

pub enum DebugLevel {
    None = 0,
    Basic = 1,
    Verbose = 2,
    Trace = 3,
}

pub fn set_level(level: DebugLevel) {
    DEBUG_LEVEL.store(level as usize, Ordering::Relaxed);
}

#[macro_export]
macro_rules! debug {
    ($level:expr, $($arg:tt)*) => {
        if $crate::debug::is_enabled($level) {
            use colored::Colorize;
            eprintln!("{} {}", "[~]".cyan().bold(), format!($($arg)*).white());
        }
    };
}

pub fn is_enabled(level: DebugLevel) -> bool {
    DEBUG_LEVEL.load(Ordering::Relaxed) >= (level as usize)
}
