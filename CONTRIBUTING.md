# Contributing to Luma

Thank you for your interest in Luma.

> **Note:** This is the reference implementation (a0.1). It will be archived once the rewrite in Luma itself (a0.2) is complete. Contributions are still welcome — bug fixes, clearer error messages, and example programs are especially useful at this stage.

---

## Ways to Contribute

- **Report a bug** — something doesn't work as expected
- **Suggest a feature** — something that would make Luma more useful or kind
- **Improve error messages** — better wording, clearer hints, more helpful suggestions
- **Write example programs** — `.lm` files that show what Luma can do
- **Fix a known issue** — see the issue tracker

---

## Reporting Bugs

Open an issue on GitHub. Please include:

1. **What you were trying to do** — a short description
2. **The Luma code that caused the problem** — paste it directly
3. **What happened** — the full error output
4. **What you expected to happen**
5. **Your OS and Rust version** — `rustc --version`

---

## Suggesting Features

Open an issue with the label `enhancement`. Describe:

- What you want to be able to do
- Why the current language makes it hard or impossible
- What you'd want the syntax to look like

Luma has a strong design philosophy — explicit, predictable, low cognitive load. Suggestions that fit that philosophy are more likely to be accepted.

---

## Setting Up for Development

### Prerequisites

- [Rust](https://rustup.rs/) stable (1.70+)
- `cargo fmt` and `cargo clippy` (included with Rust)

```bash
git clone https://github.com/dylanisaiahp/luma
cd luma
```

### Check Your Code

```bash
cargo fmt && cargo check && cargo clippy
```

---

## Project Structure

```
src/
├── main.rs
├── lib.rs
├── cli/mod.rs                   # CLI commands
├── lexer/                       # Tokenization
├── parser/                      # Parsing, AST construction
├── ast/mod.rs                   # AST node definitions
├── interpreter/                 # Evaluation
├── error/                       # Error collection and display
├── syntax/                      # Keywords and operators
└── debug/                       # Debug output
```

---

## Submitting a Pull Request

1. Fork the repo and create a branch from `main`
2. Make your changes
3. Run `cargo fmt && cargo check && cargo clippy`
4. Test your change by running a `.lm` file that exercises what you changed
5. Open a pull request with a clear description

---

## Code Style

- Follow what `cargo fmt` produces
- Prefer explicit error handling over silent failures
- Return `Result` or `Option` from fallible functions — don't panic in library code
- Error messages should be kind, specific, and actionable
- Keep files under ~200 lines where possible

---

## What Won't Be Accepted

- Features that add implicit behavior or "magic"
- Changes that make error messages more terse or technical
- Breaking changes to existing syntax without a strong reason
- Code that passes `cargo check` but fails `cargo clippy` without explanation

---

## Questions

Open an issue and ask. There are no wrong questions.
