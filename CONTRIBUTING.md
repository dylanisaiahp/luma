# Contributing to Luma

Thank you for your interest in Luma. This document covers everything you need to know to contribute — whether that's reporting a bug, suggesting a feature, or submitting code.

Luma is a project built with accessibility in mind. That applies to contributing too — if anything here is unclear or feels like a barrier, open an issue and say so.

---

## Ways to Contribute

You don't have to write code to help. Here are all the ways you can contribute:

- **Report a bug** — something doesn't work as expected
- **Suggest a feature** — something that would make Luma more useful or kind
- **Improve error messages** — better wording, clearer hints, more helpful suggestions
- **Write example programs** — `.lm` files that show what Luma can do
- **Fix a known issue** — see the issue tracker

---

## Reporting Bugs

Open an issue on GitHub. Please include:

1. **What you were trying to do** — a short description
2. **The Luma code that caused the problem** — paste it directly, don't attach a file if you can help it
3. **What happened** — the full error output
4. **What you expected to happen**
5. **Your OS and Rust version** — `rustc --version`

You don't need to follow a strict template. Clear and honest is enough.

---

## Suggesting Features

Open an issue with the label `enhancement`. Describe:

- What you want to be able to do
- Why the current language makes it hard or impossible
- What you'd want the syntax to look like (even rough ideas are helpful)

Luma has a strong design philosophy — explicit, predictable, low cognitive load. Suggestions that fit that philosophy are more likely to be accepted than ones that add magic or complexity, even if the feature itself is useful.

---

## Setting Up for Development

### Prerequisites

- [Rust](https://rustup.rs/) stable (1.70+)
- `cargo fmt` and `cargo clippy` (included with Rust)

An interactive install script is coming soon. In the meantime, clone the repo and use `cargo` directly:

```bash
git clone https://github.com/dylanisaiahp/luma
cd luma
```

### Check Your Code

Before submitting anything, run:

```bash
cargo fmt && cargo check && cargo clippy
```

All three should pass with no warnings. If clippy flags something and you disagree with it, that's fine — mention it in your pull request.

---

## Project Structure

```
src/
├── main.rs                      # Entry point
├── lib.rs                       # Library root
├── debug.rs                     # Debug macro and level control
├── cli/
│   └── mod.rs                   # CLI commands (run, check, new, --time, --debug)
├── lexer/
│   ├── mod.rs                   # Lexer exports
│   ├── core.rs                  # Tokenization loop
│   ├── tokens.rs                # Token and TokenKind definitions
│   ├── reader.rs                # Character reading helpers
│   ├── strings.rs               # String and interpolation lexing
│   ├── symbols.rs               # Operator and symbol lexing
│   ├── identifiers.rs           # Identifier and keyword lexing
│   └── comments.rs              # Comment skipping
├── parser/
│   ├── mod.rs                   # Parser exports
│   ├── core.rs                  # Parser struct, parse_program
│   ├── error.rs                 # ParseError type
│   ├── expressions.rs           # Expression entry point
│   ├── binary.rs                # Binary operator precedence parsing
│   ├── primary.rs               # Literals, identifiers, not, unary minus
│   ├── call.rs                  # Function call parsing
│   ├── assignment.rs            # Assignment and compound assignment
│   ├── statements.rs            # Statement dispatch
│   ├── declarations.rs          # Variable and function declarations
│   ├── ifelse.rs                # If / else if / else
│   ├── whileloop.rs             # While loops
│   ├── forloop.rs               # For loops with range()
│   ├── matching.rs              # Match statements
│   └── printing.rs              # Print and write statements
├── ast/
│   └── mod.rs                   # AST node definitions (Stmt, Expr, ExprKind)
├── interpreter/
│   ├── mod.rs                   # Interpreter exports and FunctionDef
│   ├── core.rs                  # Interpreter struct, execute_statement
│   ├── expressions.rs           # Expression evaluation
│   ├── operations.rs            # Binary operation dispatch
│   ├── builtins.rs              # Built-in functions (print, write, read, etc.)
│   ├── assign.rs                # Variable assignment
│   ├── statements.rs            # Statement helpers
│   ├── control.rs               # If, while, for, match, break execution
│   └── value.rs                 # Value type and RuntimeError
├── error/
│   ├── mod.rs                   # Error exports
│   ├── collector.rs             # ErrorCollector — aggregates errors and warnings
│   ├── diagnostic.rs            # Diagnostic, Span, Severity
│   ├── lexer_errors.rs          # Lexer error formatting and hints
│   ├── parse_errors.rs          # Parse error formatting and common-mistake hints
│   └── runtime_errors.rs        # Runtime error formatting and hints
└── syntax/
    ├── mod.rs                   # Syntax exports
    ├── keywords.rs              # Keyword enum and FromStr
    └── operators.rs             # BinaryOp enum
```

---

## Submitting a Pull Request

1. Fork the repo and create a branch from `main`
2. Make your changes
3. Run `cargo fmt && cargo check && cargo clippy` — fix anything that comes up
4. Test your change by running a `.lm` file that exercises what you changed
5. Open a pull request with a clear description of what changed and why

Pull requests don't need to be perfect. If you're unsure about something, submit it anyway and ask in the PR description — it's easier to review something concrete than to plan in the abstract.

---

## Code Style

- Follow what `cargo fmt` produces — don't fight it
- Prefer explicit error handling over silent failures
- If a function can fail, return a `Result` or `Option` — don't panic in library code
- Error messages should be written from the user's perspective — kind, specific, actionable
- Keep files under ~200 lines where possible — split by responsibility, not by size

---

## What Won't Be Accepted

- Features that add implicit behavior or "magic"
- Changes that make error messages more terse or technical
- Breaking changes to existing syntax without a very strong reason
- Code that passes `cargo check` but fails `cargo clippy` without explanation

---

## Questions

If you're unsure about anything — whether a bug is worth reporting, whether a feature fits the project, whether your code is good enough — just open an issue and ask. There are no wrong questions here.
