# Luma Roadmap

This document tracks what has been built and what's coming. Luma is in active early development.

---

## Versioning

- **a0.1** — Rust interpreter (current, will be archived)
- **a0.2** — Rewrite: Luma compiles itself
- **a0.3** — FFI bridge, `luma add`, package registry
- **a0.4** — `ui.lm` — windowing, rendering, input (Rust-backed)
- **a0.5** — GUI toolkit/framework built on `ui.lm`

---

## a0.1 — Rust Interpreter

### Done

- ✅ Lexer, parser, AST, interpreter
- ✅ Primitive types: `int`, `float`, `string`, `bool`, `char`
- ✅ `maybe(T)` — optional values with `.exists()` and `.or()`
- ✅ `list(T)` — typed lists with methods
- ✅ `table(K, V)` — typed key-value collections with methods
- ✅ Variable declarations with static type checking
- ✅ Proper block scoping
- ✅ If / else if / else
- ✅ While loops, for loops with `range()`, for-in loops
- ✅ Break statement
- ✅ Match statements (integer, range, string, set, `else` patterns)
- ✅ Match on `list(string)` — iterates, fires all matches
- ✅ User-defined functions with typed params and return values
- ✅ `void` function enforcement
- ✅ String interpolation (`&{var}`)
- ✅ Compound assignments (`+=`, `-=`, `*=`, `/=`)
- ✅ Logical operators (`&&`, `||`, `not`)
- ✅ Error handling: `worry(T)`, `raise`, `else error { }`
- ✅ Structs with fields and methods
- ✅ Recursion
- ✅ Multi-file projects: `use module;` — resolved by `module name;` declaration
- ✅ `luma new` — scaffolds project (git init, .gitignore, source/, luma.toml, README)
- ✅ `luma new --file name` — creates a single .lm file with module declaration
- ✅ `luma run` — runs from `luma.toml` entry if no file given
- ✅ `run()` — shell command execution, returns `worry(string)`
- ✅ Built-in functions: `print`, `write`, `read`, `input`, `int`, `float`, `string`, `random`
- ✅ `fetch(url)` — HTTP GET requests
- ✅ `file(path)` — file read/write/append/exists/list
- ✅ Rich error and warning system with common-mistake hints
- ✅ CLI: `run`, `check`, `new`, `--time`, `--debug`
- ✅ `empty` keyword for absent/uninitialized collections and optionals
- ✅ `else` as the only match wildcard (no `_`)
- ✅ Unused variable warning with correct variable name in hint

### Remaining (before archiving)

- 🔲 `luma build` — v1 compiler: AST → Rust codegen → binary

---

## a0.2 — Rewrite

Luma compiles itself. The Rust interpreter is retired and this repo is archived.

### Architecture
```
luma-core  → lexer, parser, AST, analysis, diagnostics
luma-comp  → codegen (AST → Rust), loads luma-core
luma-cli   → CLI orchestration, loads luma-comp
luma-lsp   → language server, loads luma-core
```

### Pipeline
```
.lm source
  ↓ lexer (tokens with spans)
  ↓ parser (immutable AST)
  ↓ analysis (type check, resolve)
  ↓ codegen (Rust source)
  ↓ run("rustc ...")
  ↓ binary
```

### Variable system
```luma
const int MAX = 100      # compile-time constant
int x = 5                # runtime immutable (default)
mutable int y = 5        # runtime mutable
```

### v2 CLI
```
luma new, luma init, luma run, luma build, luma check,
luma debug (levels 1-5), luma test (#T annotations),
luma tidy, luma add, luma remove, luma update, luma info,
luma search (post-website), luma install (post-website)
```

### Key principles
- AST immutable once built
- Strict phase separation
- Spans everywhere
- No generic errors
- Debug system first-class (levels 1-5)

---

## a0.3 — FFI + Packages

- `[deps]` in `luma.toml`
- `luma add github.com/user/pkg`
- Universal native bridge: `.rs`, `.py`, `.go` files in `native/`

---

## a0.4 — UI Library

- `ui.lm` — windowing, rendering, input (Rust-backed)

---

## Later / Under Discussion

- Coroutines / async
- Standard library modules (`math`, `io`, etc.)
- Generic structs
- Interfaces
- Website + docs
