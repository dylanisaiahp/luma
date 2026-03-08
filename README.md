# Luma

A small, statically-typed programming language with clear syntax and kind error messages.

```luma
void main() {
    print("Hello, Luma!");
}
```

---

## Why Luma?

Luma was built by someone who struggles to write code independently due to disabilities affecting reading, mathematics, focus, and cognitive load. Working with AI tools made building a language possible — but it also made clear how important *predictable, explicit* design is.

Luma is for anyone who finds programming languages overwhelming. It won't surprise you. It won't judge you. It'll tell you exactly what went wrong and how to fix it.

---

## Getting Started

### Install

An interactive install script is coming soon. Watch this repo for updates.

For now, clone and build with Cargo:

```bash
git clone https://github.com/dylanisaiahp/luma
cd luma
cargo build --release
```

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
luma run main.lm --debug lexer
luma run main.lm --debug parser
luma run main.lm --debug interpreter
luma run main.lm --debug all:verbose
```

### Project Structure

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
| `char` | `char c = 'a';` |
| `word` | `word tag = 'hello';` |
| `maybe(T)` | `maybe(int) val = empty;` |
| `list(T)` | `list(int) nums = (1, 2, 3);` |
| `table(K, V)` | `table(string, int) scores = ("Alice": 95);` |

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

# Functions can return lists and tables
list(int) range_list(int n) {
    list(int) result = ();
    for i in range(0, n) {
        result = result.add(i);
    }
    return result;
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

# For loop with range
for i in range(1, 6) {
    print(i);
}

# For loop over a list
for item in my_list {
    print(item);
}

# For loop over a table
for (key, value) in my_table {
    print(key);
    print(value);
}

# Break
while true {
    if done {
        break;
    }
}

# Match
match x {
    1:
        print("one");
    range(2, 5):
        print("two to five");
    _:
        print("something else");
}
```

### Error Handling

```luma
# worry(T) — a value that might fail
worry(int) safe_divide(int a, int b) {
    if b == 0 {
        raise "cannot divide by zero";
    }
    return a / b;
}

void main() {
    # Handle the error with else error
    int result = safe_divide(10, 0) else err {
        print(err);
        return;
    };
    print(result);
}
```

### Collections

```luma
# Lists
list(int) nums = (1, 2, 3, 4, 5);
print(nums.len());         # 5
print(nums.get(0));        # 1
print(nums.contains(3));   # true
list(int) sorted = nums.reverse();

# Tables
table(string, int) scores = ("Alice": 95, "Bob": 87);
print(scores.len());           # 2
print(scores.get("Alice"));    # 95
print(scores.has("Bob"));      # true
scores = scores.set("Carol", 92);
```

### Maybe

```luma
maybe(string) find_name(bool exists) {
    if exists {
        return "Dylan";
    }
    return empty;
}

void main() {
    maybe(string) name = find_name(true);
    print(name.exists());        # true
    print(name.or("Guest"));     # Dylan

    maybe(string) missing = find_name(false);
    print(missing.or("Guest"));  # Guest
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
```

### Built-in Functions

| Function | Description |
|----------|-------------|
| `print(value)` | Print a value with a newline |
| `write(value)` | Print a value without a newline |
| `read()` | Read a line from stdin |
| `input(n)` | Read the nth CLI argument |
| `int(value)` | Convert to integer |
| `float(value)` | Convert to float |
| `string(value)` | Convert to string |
| `random(min, max)` | Random integer between min and max (inclusive) |
| `fetch(url)` | Make an HTTP GET request |
| `file(path)` | Open a file handle |

### String Methods

```luma
string s = "hello world";
print(s.upper());          # HELLO WORLD
print(s.len());            # 11
print(s.contains("world")); # true
print(s.split(" "));       # list of words
```

### String Interpolation

```luma
string name = "world";
int count = 42;
print("Hello, &{name}! Count is &{count}.");
```

### File I/O

```luma
file("data.txt").write("Hello!");
string content = file("data.txt").read();
bool exists = file("data.txt").exists();
file("data.txt").append(" More text!");
```

---

## Error Messages

Luma tries to tell you exactly what went wrong and how to fix it:

```
[E010] Error: Type mismatch: expected int, got string
   ╭─[ main.lm:3:15 ]
   │
 3 │     int x = "hello";
   │             ───────
   │                ╰── Make sure the value type matches the variable declaration.
───╯
```

Warnings are less alarming on purpose:

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

## Roadmap

See [ROADMAP.md](ROADMAP.md) for what's done and what's coming.

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) or visit [github.com/dylanisaiahp](https://github.com/dylanisaiahp).

---

## License

MIT — see [LICENSE](LICENSE).
