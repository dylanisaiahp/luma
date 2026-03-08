# Luma Roadmap

This document tracks what has been built and what's coming next. Luma is in active early development — the language works, but the full feature set is still being built out.

---

## Done

- ✅ Lexer, parser, AST, interpreter
- ✅ Primitive types: `int`, `float`, `string`, `bool`
- ✅ `char` and `word` types
- ✅ Variable declarations with static type checking
- ✅ Proper block scoping
- ✅ If / else if / else
- ✅ While loops
- ✅ For loops with `range()`
- ✅ For-in loops over `list(T)`
- ✅ For-in loops over `table(K, V)`
- ✅ Break statement
- ✅ Match statements (integer, range, wildcard patterns)
- ✅ User-defined functions with typed params and return values
- ✅ `void` function enforcement (can't return a value)
- ✅ String interpolation (`&{var}`)
- ✅ Compound assignments (`+=`, `-=`, `*=`, `/=`)
- ✅ Remainder operator (`%`)
- ✅ Negative number literals
- ✅ Logical operators (`&&`, `||`, `not`)
- ✅ String equality and comparison
- ✅ `maybe(T)` type with `.exists()` and `.or()` 
- ✅ `list(T)` — typed lists with methods
- ✅ `table(K, V)` — typed key-value collections with methods
- ✅ `list(T)` and `table(K, V)` returning from functions
- ✅ `string.split()` returning `list(string)`
- ✅ Error handling: `worry(T)`, `raise`, `else error { }`
- ✅ Built-in functions: `print`, `write`, `read`, `input`, `int`, `float`, `string`, `random`
- ✅ `fetch(url)` — HTTP GET requests
- ✅ `file(path)` — file read/write/append
- ✅ Rich error and warning system with common-mistake hints
- ✅ CLI: `run`, `check`, `new`, `--time`, `--debug`

---

## In Progress / Next

- 🔲 Multi-file programs — `use module;` resolves and loads `.lm` files
- 🔲 `luma.toml` — project config (entry point, name, version)
- 🔲 Type casting between list types

---

## Later

- 🔲 Standard library modules (`math`, `io`, etc.)
- 🔲 Install script
- 🔲 More string methods
- 🔲 More collection methods
