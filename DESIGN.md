# Luma Design Document

This document captures the core design decisions and principles for the Luma rewrite (a0.2). It serves as the reference for all four repos: `luma-core`, `luma-comp`, `luma-cli`, and `luma-lsp`.

---

## Language Philosophy

- **Explicit over implicit** — if something happens, the programmer wrote it
- **Readable over terse** — full words preferred, shorthands optional and post-rewrite
- **Kind over strict** — errors should help, never confuse
- **Simple over clever** — if it needs a comment to explain, it needs a redesign
- **Type-first** — every declaration starts with its type, always

---

## Variable System

Luma has three tiers of variables:

```luma
const int MAX = 100        # compile-time constant, value known before runtime
int x = 5                  # runtime immutable, set once, never reassigned
mutable int y = 5          # runtime mutable, can change freely
```

### Rules

- `const` — value must be a literal or composed of other `const` values. Runtime calls like `random()`, `read()`, `fetch()` are illegal.
- `int x` (and all types) — immutable by default. Reassignment is a compiler error.
- `mutable` — explicit mutation. Rare in well-designed code.
- No lifetimes, no borrow syntax, no `&`, no `*` — ever.
- Mutability belongs to the **binding**, not the value.

### Memory model

- `const` values are substituted at compile time — they don't exist at runtime
- Immutable values can be stack allocated safely via escape analysis
- `mutable` values are the primary target for GC tracking
- Immutable values can be freely shared — no ownership questions
- `mutable` values have one owner — compiler enforces automatically, no annotations needed

### Future

- GC will be minimal and invisible — users never think about memory
- Immutability by default dramatically reduces GC pressure
- Long-term goal: native compilation without Rust as middleman
- Lifetimes will never be part of Luma

---

## Three Compiler Rules (from Rust/Go)

### Rule 1 — Mutability belongs to the binding
```
Symbol {
    name: string
    type: Type
    mutable: bool
    kind: Var | Const
}
```
Assignment check is just `if !symbol.mutable → error`. No complex analysis needed.

### Rule 2 — Definite initialization
Every variable must be initialized before use. The compiler tracks an `initialized` flag and performs flow analysis — all branches must initialize a variable before it can be read.

```luma
int result

if x > 10 {
    result = 1
}

print(result)   # ERR: result may not be initialized on all paths
```

### Rule 3 — Must be used
Variables, functions, imports, and constants must be used or the compiler warns. The `_` prefix suppresses the warning.

---

## Rewrite Architecture

### Four repos

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

**Rules:**
- `luma-core` depends on nothing
- `luma-comp` depends only on core
- `luma-cli` depends only on comp
- `luma-lsp` depends only on core
- No reverse dependencies ever

### Compiler pipeline

```
.lm source
  ↓ lexer        → tokens (with spans)
  ↓ parser       → immutable AST
  ↓ analysis     → type check, name resolution
  ↓ codegen      → Rust source
  ↓ run("rustc") → binary
```

### Key principles

- **AST is immutable once built** — transformations create new nodes, never mutate
- **Strict phase separation** — no phase calls into a later phase
- **Spans everywhere** — every token, AST node, and error has line/col
- **No generic errors** — every diagnostic is specific, with code, span, and hint

---

## Error System

### Error codes
- `ERR-N` for errors
- `WARN-N` for warnings
- No leading zeros, no artificial ceiling
- Every error has a unique code, specific message, span, and actionable hint

### Levels (always visible)
Errors and warnings surface on every `luma` command — they are not debug output.

### No generic errors
`ERR-16: something went wrong at runtime` will not exist in the rewrite. Every possible failure has a specific message.

---

## Debug System

`luma debug` is its own command with 5 levels:

- **Level 1** — errors only
- **Level 2** — errors + warnings
- **Level 3** — + scope changes, variable declarations
- **Level 4** — + every expression evaluated, every function call with args/return
- **Level 5** — everything: tokens, AST nodes, scope push/pop, every variable lookup

Goal: never need to add a print statement to debug Luma code.

---

## CLI

### v1 (Rust, current repo)
```
luma new      ← scaffold project (source/, luma.toml, README, .gitignore, git init)
luma run      ← interpret file or project
luma build    ← compile to binary via Rust codegen
luma check    ← parse + check, no execution
```

### v2 (post-rewrite, luma-cli in Luma)
```
luma new      ← same + improved
luma init     ← initialize existing directory
luma run      ← interpret or compile + run
luma build    ← compile to native binary
luma check    ← parse + type check
luma debug    ← debug with levels 1-5
luma test     ← run #T annotations, compare output
luma tidy     ← format + lint
luma add      ← add dependency
luma remove   ← remove dependency
luma update   ← update Luma or dependencies
luma info     ← show project info
luma search   ← search registry (post-website)
luma install  ← install components e.g. LSP (post-website)
```

---

## Test Annotations

```luma
print(add(2, 3));         #T expected output: 5
safe_divide(10, 0);       #T expected error: cannot divide by zero
```

`luma test` scans for `#T` annotations, runs the file, compares actual output.

---

## Comment System (post-rewrite)

```luma
#   regular comment
#!  critical / error
#?  warning / uncertainty  
#>  action needed (todo, fix) — syntax TBD
#T  test annotation
```

---

## Packages and Libraries

### Naming
- Official Luma libs: `namelib-lm`
- Community bindings: `name-lm`
- Community general libs: user's choice

### Bundled vs separate
- Bundled: `stdlib-lm`, `mathlib-lm`
- Separate via `luma add`: `gfxlib-lm` (interface lib), database libs, community libs

### Native bridge (a0.3)
Advanced users can place `.rs`, `.py`, `.go` etc. files in a `native/` folder alongside `.lm` files. Regular users never see this — they just `luma add libname-lm` and write Luma.

```
mylib-lm/
  source/
    api.lm           ← Luma API users call
  native/
    bindings.rs      ← Rust (or other language) internals
  luma.toml
  [native.deps]
    winit = "0.30"
```

---

## luma.toml (final spec)

```toml
[project]
name = "myapp"
version = "0.1.0"
description = ""
entry = "source/main.lm"

[deps]
http = { git = "github.com/user/luma-http" }
utils = { path = "./lib/utils" }

[dev.deps]
testhelper = { path = "./tools/testhelper" }

[native.deps]          # experimental, advanced users only
winit = { git = "https://github.com/rust-windowing/winit" }
```

---

## Repo Directory Layout

### luma-core
```
luma-core/
  source/
    lexer/
      lexer.lm
      token.lm
      exports.lm    ← module lexer;
    parser/
      parser.lm
      exports.lm    ← module parser;
    ast/
      expr.lm
      stmt.lm
      decl.lm
      exports.lm    ← module ast;
    analysis/
      symbols.lm
      types.lm
      exports.lm    ← module analysis;
    diagnostics/
      error.lm
      span.lm
      exports.lm    ← module diagnostics;
    main.lm         ← use lexer; use parser; use ast; etc.
  luma.toml
```

### luma-comp
```
luma-comp/
  source/
    loader/
      project.lm
      modules.lm
      exports.lm    ← module loader;
    codegen/
      emit_expr.lm
      emit_stmt.lm
      emit_types.lm
      emitter.lm
      exports.lm    ← module codegen;
    main.lm         ← use loader; use codegen; etc.
  luma.toml
```

### luma-cli
```
luma-cli/
  source/
    commands/
      run.lm
      build.lm
      check.lm
      new.lm
      debug.lm
      test.lm
      tidy.lm
      add.lm
      exports.lm    ← module commands;
    main.lm         ← use commands;
  luma.toml
```

### luma-lsp
```
luma-lsp/
  source/
    protocol/
      message.lm
      exports.lm    ← module protocol;
    features/
      diagnostics.lm
      hover.lm
      completion.lm
      exports.lm    ← module features;
    main.lm         ← use protocol; use features;
  luma.toml
```
