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
- ✅ Match on `list(string)` — iterates, fires all matches
- ✅ User-defined functions with typed params and return values
- ✅ `void` function enforcement
- ✅ String interpolation (`&{var}`)
- ✅ Compound assignments (`+=`, `-=`, `*=`, `/=`)
- ✅ Logical operators (`&&`, `||`, `not`)
- ✅ Error handling: `worry(T)`, `raise`, `else error { }`
- ✅ Structs with fields and methods
- ✅ Recursion (scope leak fix — if/while/for/for-in all correct)
- ✅ Multi-file projects: `use module;`, `module name;`, `use X as Y` (pending)
- ✅ `luma.toml` — project config with `[project]` and `entry`
- ✅ `luma run` — runs from `luma.toml` entry if no file given
- ✅ `luma new` — scaffolds project with `source/`, `luma.toml`, `README.md`
- ✅ `run()` — shell command execution, returns `worry(string)`
- ✅ Built-in functions: `print`, `write`, `read`, `input`, `int`, `float`, `string`, `random`
- ✅ `fetch(url)` — HTTP GET requests
- ✅ `file(path)` — file read/write/append/exists
- ✅ Rich error and warning system with common-mistake hints
- ✅ CLI: `run`, `check`, `new`, `--time`, `--debug`
- ✅ Reorganized test suite (`tests/types`, `collections`, `control`, `functions`, `io`, `errors`, `structs`)

### Remaining (before rewrite)

#### Language features
- 🔲 `for c in string` — character iteration
- 🔲 `list(string).join(separator)` — needed for codegen
- 🔲 `file().files()` / `file().dirs()` — directory traversal, replaces `dir()`
- 🔲 `cli_args()`, `cli_flags()`, `cli_opts()` — replaces `input()` redesign
- 🔲 `use X as Y` — import aliasing
- 🔲 Generics: `generic` keyword for functions and structs
- 🔲 Module conflict error (`http.lm` + `http/` both exist)
- 🔲 `use http.client` single import fix (already partially done)

#### Comment system expansion
- 🔲 `#` — regular comment (done)
- 🔲 `#!` — critical/error comment
- 🔲 `#?` — warning/uncertainty comment
- 🔲 `#>` — action needed (todo, fix)
- 🔲 `#T expected output: value` — test annotation
- 🔲 `#T expected error: message` — test annotation

#### CLI (v1 — Rust)
- 🔲 `luma build` — compile to binary via Rust codegen (v1 compiler)
- 🔲 `luma new` — add git init + .gitignore

#### Error + debug system (major improvement for rewrite)
- 🔲 No generic errors — every error specific, with span, hint, code
- 🔲 `luma debug` as own command (not a flag) with multiple levels
- 🔲 Level 1: errors only
- 🔲 Level 2: errors + warnings
- 🔲 Level 3: + scope changes
- 🔲 Level 4: + every expression evaluated
- 🔲 Level 5: everything — tokens, AST, scope push/pop, variable lookups, calls
- 🔲 Never need `eprintln!` for debugging again

#### v1 Compiler (Rust, inside current repo)
- 🔲 AST → Rust codegen
- 🔲 `luma build` produces native binary
- 🔲 Spans propagate through all phases

---

## a0.2 — Rewrite

Luma compiles itself. The Rust interpreter is retired.

### Architecture
```
luma-core  → lexer, parser, AST, analysis, diagnostics
luma-comp  → codegen (AST → Rust), loads luma-core
luma-cli   → CLI orchestration, loads luma-comp
luma-lsp   → language server, loads luma-core
```

### Dependency graph
```
luma-core
   ↑      ↑
luma-comp  luma-lsp
   ↑
luma-cli
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

### v2 CLI (post-rewrite)
```
luma new        ← scaffold project (git init, .gitignore, source/, luma.toml)
luma init       ← initialize existing directory as Luma project
luma run        ← interpret file or project
luma build      ← compile to native binary
luma check      ← parse + type check, no execution
luma debug      ← debug with levels (1-5)
luma test       ← run #T annotations, compare output
luma tidy       ← format + lint (fmt + clippy equivalent)
luma add        ← add dependency to luma.toml
luma remove     ← remove dependency
luma update     ← update Luma or dependencies
luma info       ← show project/dependency info
luma search     ← search package registry (post-website)
luma install    ← install Luma components e.g. LSP (post-website)
```

### Key design principles
- AST is immutable once built — transformations create new nodes
- Strict phase separation — no phase calls into a later phase
- Spans everywhere — every token, AST node, and error has line/col
- No generic errors — every diagnostic is specific and actionable
- Debug system is first-class, not bolted on

---

## a0.3 — FFI + Packages

- `[deps]` in `luma.toml` — Luma package dependencies
- `[dev.deps]` — development-only dependencies
- `[rust.deps]` — raw Rust crate dependencies (experimental, advanced users)
- `luma add github.com/user/pkg` — auto-detects git/local/network
- Package naming convention: `name-lm` (repo), `name.lm` (import)
- Dependency sources: `{ git = }`, `{ path = }`, `{ network = }`
- `luma-converter` — `luma convert file.rs/.py/.go` (mini compiler, no binary)

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
- Website + docs (luma-lang.io?)
- More string and collection methods
- IR layer (post-rewrite, sits between AST and codegen)
- `luma search` / `luma install` (post-website)
