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

## Locked Decisions (finalized in a0.1 cleanup)

These are not up for discussion in the rewrite — they are locked.

- `word` type removed — use `string` or `char`
- `_` wildcard removed from match — use `else` only
- `()` no longer valid for empty list/table init — use `empty` keyword only
- Module system: every importable `.lm` file must declare `module name;` — no filename fallback, module name CAN match filename
- `luma new --file name` creates a `.lm` file pre-populated with `module name;`
- `else` is the only match wildcard — patterns are: integer, range, string, set, else
- Single-quote literals produce `char` only

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

print(result)   # Err:Uninit-Var — result may not be initialized on all paths
```

### Rule 3 — Must be used
Variables, functions, imports, and constants must be used or the compiler warns. The `_` prefix suppresses the warning.

---

## Types

```luma
int           # 64-bit signed integer
float         # 64-bit float
string        # UTF-8 text
bool          # true / false
char          # single Unicode character, single-quote literal 'a'
maybe(T)      # optional value — .exists() / .or(fallback)
worry(T)      # failable value — raise / else error { }
list(T)       # ordered collection
table(K, V)   # key-value collection
```

`maybe(T)?` shorthand is planned post-rewrite but not in a0.1.

### Collections

- `list(T)` — immutable by default. `.add()` returns a new list, it does not mutate in place.
- `table(K, V)` — same immutability model. More methods in rewrite.
- `empty` — the only way to initialize an empty list or table. `()` is not valid.

---

## Method APIs

### String / Char

`.len()` `.upper()` `.lower()` `.trim()` `.reverse()` `.chars()` `.contains()` `.starts_with()` `.ends_with()` `.repeat()` `.replace()` `.split()` `.first()` `.last()` `.as(type)` `.exists()`

### List

`.len()` `.get(i)` `.contains()` `.where()` `.add()` `.remove(i)` `.reverse()` `.sort()` `.first()` `.last()` `.merge(glue)` `.exists()`

### Table

`.len()` `.get(key)` `.has(key)` `.set(key, val)` `.remove(key)` `.keys()` `.values()` `.exists()`

### input()

```luma
list(string) args  = input().args();
list(string) flags = input().flags();
table(string, string) opts = input().options();
```

### file()

```luma
file("path").read()
file("path").write("content")
file("path").append("content")
file("path").exists()
file("path/").list(".")       # all files
file("path/").list(".lm")     # by extension
file("path/").list("/")       # dirs only
file("path/").list("/sub/")   # subdir contents
```

### fetch()

```luma
fetch("https://...").get()
fetch("https://...").send("body")
```

---

## Error and Warning System

### Design goals

The error system in a0.1 was a foundation. The rewrite replaces it completely with these goals:

- **No generic errors** — every diagnostic has a specific code, message, span, and actionable hint. A fallback "something went wrong" message will not exist.
- **Cross-file diagnostics** — errors must show the correct file, line, and column regardless of which file in the module chain triggered them. The current a0.1 implementation can fail silently, point at a `use` statement rather than the actual error site, or produce a Rust backtrace. None of these are acceptable.
- **Every error is recoverable** — the compiler collects all errors before reporting, never stops at the first one.
- **Errors and warnings always surface** — on every `luma` command, not just `luma check`.
- **Hints are prescriptive** — they tell the programmer exactly what to change, not just what went wrong.

### Diagnostic codes

Codes use an `Err:` or `Warn:` prefix followed by a short title-case slug. This keeps them self-documenting without being verbose, and title case means Luma is talking to you, not shouting.

```
[Err:Type-Mismatch]
[Err:Undef-Var]
[Err:Convert-Fail]
[Err:Missing-Semi]
[Err:Uninit-Var]
[Err:Div-By-Zero]
[Err:Unknown-Func]
[Err:Wrong-Arg-Count]
[Err:Not-A-Bool]
[Err:Unterminated-String]
[Warn:Unused-Var]
[Warn:Unused-Import]
[Warn:Unused-Func]
[Warn:Void-Returns-Value]
```

The format is locked. The specific slugs are not — if a slug is unclear or gets complaints it can be renamed without touching diagnostic logic.

### Diagnostic appearance

Each diagnostic renders in layers with distinct colors, designed to feel calm and teacherly rather than alarming:

```
[Err:Type-Mismatch] Type mismatch: expected int, got string    ← pastel red
   --> main.lm:3:9                                             ← cyan

 3 │   int x = "hello";                                        ← neutral/dimmed
           ~~~~~~~
           Use a string variable, or change the type to string. ← soft blue or muted green
```

```
[Warn:Unused-Var] Unused variable: 'result'                    ← warm light yellow
   --> main.lm:5:9                                             ← cyan

 5 │   int result = 0;                                         ← neutral/dimmed
           ~~~~~~
           If you meant to ignore it, prefix with underscore: _result ← soft blue or muted green
```

Color intent:
- **Error label + message** — pastel red. Noticeable but not aggressive.
- **Warning label + message** — warm light yellow. Informational, not urgent.
- **File/location arrow** — cyan.
- **Source line** — neutral, dimmed. Context, not the focus.
- **Hint** — soft blue or muted green. Feels like a suggestion, not a command.

The color scheme is the current direction, not a hard lock. The palette lives in one place in `luma-core/diagnostics` and can be adjusted without touching any diagnostic logic.

### Diagnostic shape

Every diagnostic carries:
- Code — `Err:Slug` or `Warn:Slug`
- Severity
- Message — what went wrong, in plain language
- Span — filename, line, column, length (always accurate, even across files)
- Source line — the actual text at that location
- Hint — what to do about it, prescriptive and specific

### Cross-file requirement

When a module is `use`d and contains an error, the diagnostic must point to the exact location in the module file — not to the `use` statement in the importing file. The `luma-core` diagnostics module owns all span tracking and must thread spans through the entire pipeline so that all errors always resolve to their true source location.

### Always visible

Errors and warnings surface on every `luma` command — they are not debug output.

---

## Debug System

`luma debug` is its own command with 5 levels:

- **Level 1** — errors only
- **Level 2** — errors + warnings
- **Level 3** — + scope changes, variable declarations
- **Level 4** — + every expression evaluated, every function call with args/return
- **Level 5** — everything: tokens, AST nodes, scope push/pop, every variable lookup

Goal: never need to add a print statement to debug Luma code.

The debug system must be cross-file aware — when tracing execution across module boundaries, the debug output shows which file each event came from.

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
- **Spans everywhere** — every token, AST node, and error has file + line + col
- **No generic errors** — every diagnostic is specific, with code, span, and hint
- **Cross-file correctness** — spans always resolve to the actual source file, never to an import site

---

## CLI

### v1 (Rust, current repo)
```
luma new               ← done (git init + .gitignore + source/ + luma.toml + README)
luma new --file name   ← done (creates name.lm with module name; declaration)
luma run               ← done
luma check             ← done
luma build             ← TODO (v1 compiler: AST → Rust codegen → binary via rustc)
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
#>  action needed (todo, fix)
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
