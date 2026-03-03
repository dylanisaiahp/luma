# Luma 🌙

A small programming language designed to be **kind** — explicit, predictable, and gentle on the brain.

Luma was built for people who want to focus on *what* they're building, not fight their tools. Every design decision prioritizes clarity over cleverness.

---

## Philosophy

- **No hidden magic** — everything is visible and explicit
- **Predictable behavior** — what you see is what you get
- **Kind errors** that explain *how* to fix problems, not just *that* something broke
- **Low cognitive load** — consistent patterns, minimal punctuation noise
- **Multiple errors at once** — never stop at the first problem and leave you guessing

---

## What It Looks Like
```luma
void main() {
    int x = 5;
    string name = "Luma";

    if x > 3 {
        print("x is greater than 3");
    }

    while x > 0 {
        print("&{x} remaining");
        x -= 1;
    }

    print("Hello, &{name}!");
}
```

---

## Key Features

- **Explicit types** — `int x = 5;`, `string name = "Luma";`, `bool done = false;`
- **No sigils** — just clear, readable keywords
- **String interpolation** — `"Hello, &{name}!"` just works
- **Logical operators** — `&&` and `||`
- **Match statements** with integer and range patterns
- **Compound assignments** — `+=`, `-=`, `*=`, `/=`
- **Proper block scoping** — variables don't leak outside their block
- **Kind error messages** with source snippets, `^` pointers, and fix suggestions
- **Warnings** for unused variables with actionable hints
- **Debug levels** — `--debug basic`, `--debug verbose`, `--debug trace`
- **Fast** — runs simple programs in ~200 microseconds

---

## Error Messages

Luma's errors are designed to be friendly, not terse:
```
[E001] Error: Missing semicolon
   ╭─[ main.lm:3:14 ]
   │
 3 │     int x = 5
   │              │
   │              ╰─ Suggestion: Add a semicolon at the end of this statement.
───╯
```

Warnings use `[?]` instead of `[!]` so they feel less alarming:
```
[?] Warning: Unused variable: 'x'
   ╭─[ main.lm:2:9 ]
   │
 2 │     int x = 5;
   │         │
   │         ╰─ If you meant to ignore it, prefix with underscore: _x
───╯
```

---

## Getting Started

### Install

An interactive install script is coming soon. Watch this repo for updates.

### Usage
```bash
# Run a Luma file
luma run main.lm

# Check for errors without running
luma check main.lm

# Create a new project
luma new myproject

# Show execution time
luma run main.lm --time

# Enable debug output
luma run main.lm --debug basic
luma run main.lm --debug verbose
luma run main.lm --debug trace
```

### Project Structure

A Luma project looks like this:
```
myproject/
├── lm/
│   └── main.lm
└── README.md
```

---

## Language Reference

### Types

| Type | Example |
|------|---------|
| `int` | `int x = 42;` |
| `float` | `float pi = 3.14;` |
| `string` | `string name = "Luma";` |
| `bool` | `bool done = false;` |

### Control Flow
```luma
# If / else if / else
if x > 10 {
    print("big");
} else if x > 5 {
    print("medium");
} else {
    print("small");
}

# While loop
while x > 0 {
    x -= 1;
}

# Match statement
match x {
    1:
        print("one");
    range(2, 5):
        print("two to five");
    _:
        print("something else");
}
```

### Operators
```luma
# Arithmetic
x + y    x - y    x * y    x / y

# Comparison
x == y   x != y   x > y   x < y   x >= y   x <= y

# Logical
x && y   x || y

# Compound assignment
x += 1   x -= 1   x *= 2   x /= 2
```

### Built-in Functions

| Function | Description |
|----------|-------------|
| `print(value)` | Print a value to stdout |
| `read()` | Read a line from stdin |
| `int(value)` | Convert to integer |
| `float(value)` | Convert to float |
| `string(value)` | Convert to string |
| `random(min, max)` | Random integer between min and max (inclusive) |

### String Interpolation
```luma
string name = "world";
int count = 42;
print("Hello, &{name}! Count is &{count}.");
```

---

## Current State

Luma is in **active early development**. It works, but some features are still being built.

### Working
- ✅ Lexer, parser, AST, interpreter
- ✅ All four primitive types (int, float, string, bool)
- ✅ Variable declarations with type checking
- ✅ Proper block scoping
- ✅ If / else if / else
- ✅ While loops
- ✅ Match statements with integer and range patterns
- ✅ String interpolation
- ✅ Compound assignments
- ✅ Logical operators (`&&`, `||`)
- ✅ String equality and comparison
- ✅ Built-in functions (print, read, int, float, string, random)
- ✅ Kind error and warning system
- ✅ CLI (run, check, new, --time, --debug)

### Coming Soon
- 🔲 User-defined functions with return values
- 🔲 `for` loops with `range()`
- 🔲 Method chaining (`.`)
- 🔲 More built-in functions
- 🔲 More types (char, list, array, table)

---

## Why Luma?

Luma was built by someone who struggles to write code independently due to disabilities affecting reading, mathematics, focus, and cognitive load. Working with AI tools made building a language possible — but it also made clear how important *predictable, explicit* design is.

Luma is for anyone who finds programming languages overwhelming. It won't surprise you. It won't judge you. It'll tell you exactly what went wrong and how to fix it.

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) or visit [github.com/dylanisaiahp](https://github.com/dylanisaiahp).

---

## License

MIT — see [LICENSE](LICENSE).
