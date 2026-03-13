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
cp target/release/luma ~/.local/bin/luma
```

### Usage

```bash
luma new myproject     # Create a new project
luma run               # Run from luma.toml entry point
luma run main.lm       # Run a specific file
luma check main.lm     # Check for errors without running
luma run --time        # Show execution time
luma run --debug all   # Debug output
```

### Project Structure

```
myproject/
├── source/
│   └── main.lm
├── luma.toml
└── README.md
```

```toml
[project]
name = "myproject"
version = "0.1.0"
description = ""
entry = "source/main.lm"
```

---

## Examples

### Hello World

```luma
void main() {
    print("Hello, Luma!");
}
```

### Functions

```luma
int add(int x, int y) {
    return x + y;
}

void main() {
    print(add(3, 4));   # 7
}
```

### Control Flow

```luma
void main() {
    for i in range(1, 6) {
        if i % 2 == 0 {
            print("&{i} is even");
        }
    }
}
```

### Error Handling

```luma
worry(int) safe_divide(int a, int b) {
    if b == 0 {
        raise "cannot divide by zero";
    }
    return a / b;
}

void main() {
    int result = safe_divide(10, 0) else err {
        print(err);
        return;
    };
    print(result);
}
```

### Structs

```luma
struct Point {
    int x;
    int y;

    int sum() {
        return x + y;
    }
}

void main() {
    Point p = Point(x: 3, y: 7);
    print(p.x);       # 3
    print(p.sum());   # 10
}
```

### Multi-file Projects

```luma
# source/greet.lm
string greet(string name) {
    return "Hello, &{name}!";
}
```

```luma
# source/main.lm
use greet;

void main() {
    print(greet("Luma"));
}
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

---

## Roadmap

See [ROADMAP.md](ROADMAP.md) for what's done and what's coming.

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) or visit [github.com/dylanisaiahp](https://github.com/dylanisaiahp).

---

## License

MIT — see [LICENSE](LICENSE).
