// src/debug/config.rs

#[derive(Debug, Clone, Default)]
pub struct DebugConfig {
    pub lexer: bool,
    pub parser: bool,
    pub interpreter: bool,
    pub verbose: bool,
}

impl DebugConfig {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn from_flags(flags: &[&str]) -> Self {
        let mut config = Self::none();
        for flag in flags {
            match *flag {
                "all" => {
                    config.lexer = true;
                    config.parser = true;
                    config.interpreter = true;
                }
                "all:verbose" => {
                    config.lexer = true;
                    config.parser = true;
                    config.interpreter = true;
                    config.verbose = true;
                }
                "lexer" => config.lexer = true,
                "lexer:verbose" => {
                    config.lexer = true;
                    config.verbose = true;
                }
                "parser" => config.parser = true,
                "parser:verbose" => {
                    config.parser = true;
                    config.verbose = true;
                }
                "interpreter" => config.interpreter = true,
                "interpreter:verbose" => {
                    config.interpreter = true;
                    config.verbose = true;
                }
                _ => eprintln!("Unknown debug flag: '{}'", flag),
            }
        }
        config
    }

    #[allow(dead_code)]
    pub fn any(&self) -> bool {
        self.lexer || self.parser || self.interpreter
    }
}
