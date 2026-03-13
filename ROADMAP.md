# Luma Roadmap

This document tracks what has been built and what's coming. Luma is in active early development.

---

## Versioning

- **a0.1** — Rust interpreter (current)
- **a0.2** — Rewrite: Luma compiles itself
- **a0.3** — FFI bridge, `luma add`, package registry
- **a0.4** — `ui.lm` — windowing, rendering, input (Rust-backed)
- **a0.5** — GUI toolkit/framework built on `ui.lm`

---

## a0.1 — Rust Interpreter

### Done

- ✅ Lexer, parser, AST, interpreter
- ✅ Primitive types: `int`, `float`, `string`, `bool`, `char`, `word`
- ✅ `maybe(T)` — optional values with `.exists()` and `.or()`
- ✅ `list(T)` — typed lists with methods
- ✅ `table(K, V)` — typed key-value collections with methods
- ✅ Variable declarations with static type checking
- ✅ Proper block scoping
- ✅ If / else if / else
- ✅ While loops, for loops with `range()`, for-in loops
- ✅ Break statement
- ✅ Match statements (integer, range, wildcard, string, set patterns)
- ✅ User-defined functions with typed params and return values
- ✅ `void` function enforcement
- ✅ String interpolation (`&{var}`)
- ✅ Compound assignments (`+=`, `-=`, `*=`, `/=`)
- ✅ Logical operators (`&&`, `||`, `not`)
- ✅ Error handling: `worry(T)`, `raise`, `else error { }`
- ✅ Structs with fields and methods
- ✅ Multi-file projects: `use module;`, `module name;`
- ✅ `luma.toml` — project config with `[project]` and `entry`
- ✅ `luma run` — runs from `luma.toml` entry if no file given
- ✅ `luma new` — scaffolds project with `source/`, `luma.toml`, `README.md`
- ✅ Built-in functions: `print`, `write`, `read`, `input`, `int`, `float`, `string`, `random`
- ✅ `fetch(url)` — HTTP GET requests
- ✅ `file(path)` — file read/write/append
- ✅ Rich error and warning system with common-mistake hints
- ✅ CLI: `run`, `check`, `new`, `--time`, `--debug`

### Remaining

- 🔲 `run()` — execute a string of Luma code at runtime
- 🔲 `input()` redesign
- 🔲 `luma add` — add a dependency
- 🔲 `luma build` — compile to binary via Rust codegen
- 🔲 Module conflict error (`http.lm` + `http/` both exist)
- 🔲 `use http.client` single import (no parens needed)
- 🔲 `use X as Y` — import aliasing
- 🔲 Generics: `generic` keyword for functions and structs
- 🔲 Install script

---

## a0.2 — Rewrite

Luma compiles itself. The Rust interpreter is retired.

- `luma-core` — core language in Luma
- `luma-comp` — compiler in Luma (`lumac`)
- `luma-cli` — CLI in Luma
- `luma-lsp` — language server in Luma

---

## a0.3 — FFI + Packages

- `[deps]` in `luma.toml` — Luma package dependencies
- `[dev.deps]` — development-only dependencies
- `[rust.deps]` — raw Rust crate dependencies (experimental, advanced users)
- `luma add github.com/user/pkg` — auto-detects git/local/network
- Package naming convention: `name-lm` (repo), `name.lm` (import)
- Dependency sources: `{ git = }`, `{ path = }`, `{ network = }`

---

## a0.4 — UI Library

- `ui.lm` — official windowing, rendering, input library (Rust-backed)
- User choice: CPU or GPU rendering
- User choice: retained or immediate mode
- Fonts, input handling, window management

---

## Later / Under Discussion

- Coroutines / async
- Standard library modules (`math`, `io`, etc.)
- Generic structs
- Interfaces / implementing (`: Interface` syntax)
- `luma-lsp` — language server for editor support
- Website + docs
- More string and collection methods
