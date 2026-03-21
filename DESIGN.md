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
- Module system: every importable `.lm` file must declare `module name;` — no filename fallback
- `luma create path/to/file` creates a `.lm` file pre-populated with `module name;`
- `luma create path/to/dir/` (trailing slash) creates a directory
- `else` is the only match wildcard — patterns are: integer, range, string, set, ok, error, else
- Single-quote literals produce `char` only
- `and`, `or`, `not` are keywords — `&&`, `||`, `!` (standalone) are not valid Luma
- `!=` stays — it is a comparison operator, not standalone negation
- `else e {}` error handling syntax removed — replaced by `worry` match (see below)

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

## Types

```luma
int           # 64-bit signed integer
float         # 64-bit float
string        # UTF-8 text
bool          # true / false
char          # single Unicode character, single-quote literal 'a'
generic       # any type — explicit, replaces "any" from other languages
maybe(T)      # optional value — .exists() / .or(fallback)
worry(T)      # failable value — match ok/error
list(T)       # ordered collection
table(K, V)   # key-value collection
```

### `generic` type

`generic` is explicit — you declare it deliberately when a value genuinely needs to work with any type. It is not a fallback or an escape hatch.

```luma
generic identity(generic value) {
    return value;
}

list(generic) mixed = (1, "hello", true);
```

Combined with `is`:
```luma
generic val = something();
if val is int {
    print(val + 1);
} else if val is string {
    print(val.upper());
}
```

### `is` keyword

`is` checks the type of any value. Works on primitives, structs, `ok`, `error`, `maybe`, `generic` — everything.

```luma
if x is int {
if name is string {
if result is ok {
if result is error {
if value is maybe {
```

`.is(type)` also works as a method — both forms are valid:

```luma
if x.is(int) {
if result.is(ok) {
```

---

## Error Handling

### `worry(T)`

A `worry(T)` is either ok with a value of type `T`, or an error with a string message. There is no magic conversion — if a function returns `worry(int)`, the result is a `worry(int)` until explicitly handled.

```luma
worry(int) safe_divide(int a, int b) {
    if b == 0 {
        raise "cannot divide by zero";
    }
    return a / b;
}
```

### Handling with match

```luma
worry(int) result = safe_divide(10, 2);

match result {
    ok: print(result.value);
    error: print(result.message);
}
```

- `ok` and `error` are match patterns specific to `worry` types
- `result.value` — the inner value. Runtime error if called on error state: "`.value` can't be called on error — check `is ok` first"
- `result.message` — the error string. Runtime error if called on ok state
- `else` is optional for `worry` match — `ok` and `error` are exhaustive
- Compiler warns if neither `ok` nor `error` is handled

### Checking with `is`

```luma
if result is ok {
    print(result.value);
}
if result is error {
    print(result.message);
}
```

### The old `else e {}` syntax is removed

The previous syntax was implicit and inconsistent. `worry` match replaces it entirely.

---

## Method APIs

### String / Char

`.len()` `.upper()` `.lower()` `.trim()` `.reverse()` `.chars()` `.contains()` `.starts_with()` `.ends_with()` `.repeat()` `.replace()` `.split()` `.first()` `.last()` `.as(type)` `.exists()`

### int / float

`.abs()` `.to_float()` / `.to_int()` `.to_string()` `.exists()` `.pow()` `.floor()` `.ceil()` `.round()`

`.with(format)` — format as a display string, always returns `string`. Lives on `int` and `float` only — not on `string`. Formatting is a numeric concern.

```luma
int price = 1299;
price.with("$");     # "$1,299"
price.with(",");     # "1,299"
price.with("€");     # "€1,299"

float bytes = 1073741824.0;
bytes.with("GB");    # "1.00 GB"
bytes.with("MB");    # "1,024 MB"
```

### List

`.len()` `.get(i)` `.contains()` `.where()` `.add()` `.remove(i)` `.reverse()` `.sort()` `.first()` `.last()` `.merge(glue)` `.exists()`

### Table

`.len()` `.get(key)` `.has(key)` `.set(key, val)` `.remove(key)` `.keys()` `.values()` `.exists()`

### maybe(T)

`.exists()` `.or(fallback)` `.is(type)`

### worry(T)

`.value` — inner value (ok only)
`.message` — error string (error only)
`.is(ok)` `.is(error)`

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
```

### fetch()

```luma
fetch("https://...").get()
fetch("https://...").send("body")
```

---

## Match Statements

Patterns: integer, range, string, set, `ok`, `error`, `else`

```luma
# scalar match — else required
match val {
    1: print("one");
    range(2, 5): print("small");
    ("quit", "exit"): print("goodbye");
    else: print("other");
}

# worry match — else optional, ok/error are exhaustive
match result {
    ok: print(result.value);
    error: print(result.message);
}

# list match — fires all matching arms per element
match flags {
    ("verbose", "v"): print("verbose on");
    ("time", "t"): print("time on");
    else: print("unknown flag");
}
```

---

## Rewrite Architecture

### Repo structure

```
luma-lang/
    core/        ← AST, lexer, parser, analysis, diagnostics (depends on nothing)
    comp/        ← compiler, AST → Rust → binary (depends on core)
    interface/   ← CLI commands (depends on comp)
    lsp/         ← language server (depends on core only)
    bridge/      ← universal bindings for other languages
    migrate/     ← version migration between Luma versions
    weave/       ← convert to/from other languages
    examples/
```

### Dependency graph

```
core
  ↑    ↑    ↑      ↑       ↑
comp  lsp  bridge migrate weave
  ↑
interface
```

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
- **Cross-file correctness** — spans always resolve to the actual source file

---

## Error System

### Error codes
- `ERR-N` for errors
- `WARN-N` for warnings
- No leading zeros, no artificial ceiling
- Every error has a unique code, specific message, span, and actionable hint

### No generic errors
`ERR-16: something went wrong at runtime` will not exist in the rewrite. Every possible failure has a specific message.

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

---

## CLI

### v1 (Rust, current repo)
```
luma new <name>       ← scaffold project
luma create <path>    ← create .lm file or directory
luma run              ← interpret from luma.toml entry
luma run <file>       ← interpret specific file
luma build            ← compile to binary (TODO)
luma check <file>     ← parse + check, no execution
```

### v2 (post-rewrite, luma-cli in Luma)
```
luma / luma help          ← full help
luma help <command>       ← help for specific command
luma docs                 ← print docs URL, prompt for topic
luma docs <topic>         ← reference for that topic (read, errors, worry, etc.)
luma new                  ← scaffold project
luma init                 ← initialize existing directory
luma create               ← create .lm file or directory
luma run                  ← interpret or compile + run
luma build                ← compile to native binary
luma check                ← parse + type check
luma debug                ← debug with levels 1-5
luma test                 ← run #T annotations, compare output
luma tidy                 ← format + lint
luma add                  ← add dependency
luma remove               ← remove dependency
luma update               ← update Luma or dependencies
luma info                 ← show project info
luma migrate              ← migrate project between Luma versions
luma weave                ← convert to/from other languages
luma search               ← search registry (post-website)
luma install              ← install components e.g. LSP (post-website)
```

### Documentation tiers

Luma has two tiers of documentation:

**Docs** — verbose, explanatory, for learning. Full prose, examples, context.

**Reference** — terse, scannable, for when you already know the language:

```
read() — reads a line from stdin, returns string

  string line = read();

  .trim()    → string   remove leading/trailing whitespace
  .upper()   → string   uppercase
  .lower()   → string   lowercase
  .split()   → list     split on whitespace
```

`luma docs read` prints the reference entry for `read`. `luma docs` prints the website URL and prompts for a topic.

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
    api.lm
  native/
    bindings.rs
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

[native.deps]
winit = { git = "https://github.com/rust-windowing/winit" }
```

---

## Editor Support

The current repo contains a tree-sitter grammar and Zed extension as a development aid for the rewrite period only. These are not maintained and will be retired when `luma-lang/lsp/` is complete.

The LSP (`luma-lang/lsp/`) replaces all tree-sitter grammars — one language server works in Zed, VS Code, Neovim, Helix, and everything else.

---

## Repo Directory Layout

### luma-lang/core/
```
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
    main.lm
luma.toml
```

### luma-lang/comp/
```
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
    main.lm
luma.toml
```

### luma-lang/interface/
```
source/
    commands/
        run.lm
        build.lm
        check.lm
        new.lm
        create.lm
        debug.lm
        test.lm
        tidy.lm
        add.lm
        docs.lm
        migrate.lm
        weave.lm
        exports.lm    ← module commands;
    main.lm
luma.toml
```

### luma-lang/lsp/
```
source/
    protocol/
        message.lm
        exports.lm    ← module protocol;
    features/
        diagnostics.lm
        hover.lm
        completion.lm
        exports.lm    ← module features;
    main.lm
luma.toml
```
