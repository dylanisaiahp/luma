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

    for i in range(1, 4) {
        write("&{i}... ");
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
- **For loops** — `for i in range(1, 10) { ... }`
- **Match statements** with integer, range, and wildcard patterns
- **User-defined functions** with typed parameters and return values
- **Logical operators** — `&&`, `||`, and `not`
- **Compound assignments** — `+=`, `-=`, `*=`, `/=`
- **Remainder operator** — `x % y`
- **Negative literals** — `-42`, `-3.14`
- **Proper block scoping** — variables don't leak outside their block
- **Kind error messages** — source snippets, pointers, fix suggestions, and common-mistake hints
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

Luma also recognizes common mistakes from other languages:
```
[E002] Error: I don't know what 'println' means here
   ╭─[ main.lm:2:5 ]
   │
 2 │     println("hello");
   │     │
   │     ╰─ Luma uses print() — it already adds a newline automatically.
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

### Functions
```luma
# Void function — no return value
void greet(string name) {
    print("Hello, &{name}!");
}

# Typed function — returns a value
int add(int x, int y) {
    return x + y;
}

void main() {
    greet("world");
    int result = add(3, 4);
    print(result);
}
```

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

# For loop
for i in range(1, 6) {
    print(i);
}

# Break out of a loop
while true {
    if done {
        break;
    }
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
x + y    x - y    x * y    x / y    x % y

# Comparison
x == y   x != y   x > y   x < y   x >= y   x <= y

# Logical
x && y   x || y   not x

# Compound assignment
x += 1   x -= 1   x *= 2   x /= 2

# Negative literals
int n = -42;
float f = -3.14;
```

### Built-in Functions

| Function | Description |
|----------|-------------|
| `print(value)` | Print a value followed by a newline |
| `write(value)` | Print a value without a newline |
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

### Imports (stub)
```luma
use math;
```
Full import system and standard library coming soon.

---

## Current State

Luma is in **active early development**. It works, but some features are still being built.

### Working
- ✅ Lexer, parser, AST, interpreter
- ✅ All four primitive types (`int`, `float`, `string`, `bool`)
- ✅ Variable declarations with type checking
- ✅ Proper block scoping
- ✅ If / else if / else
- ✅ While loops
- ✅ For loops with `range()`
- ✅ Break statement
- ✅ Match statements with integer, range, and wildcard patterns
- ✅ User-defined functions with typed params and return values
- ✅ String interpolation
- ✅ Compound assignments (`+=`, `-=`, `*=`, `/=`)
- ✅ Remainder operator (`%`)
- ✅ Negative number literals
- ✅ Logical operators (`&&`, `||`, `not`)
- ✅ String equality and comparison
- ✅ Built-in functions (`print`, `write`, `read`, `int`, `float`, `string`, `random`)
- ✅ Kind error and warning system with common-mistake hints
- ✅ `use` keyword (stub — imports coming soon)
- ✅ CLI (`run`, `check`, `new`, `--time`, `--debug`)

### Coming Soon
- 🔲 Method chaining (`.`)
- 🔲 Extension methods (`int.squared()`, `string.shout()`)
- 🔲 `maybe()` type with `.or()` for optional values
- 🔲 New types (`char`, `list`, `table`)
- 🔲 Standard library (`math`, `io`, etc.)
- 🔲 Import system + `luma.toml`

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
