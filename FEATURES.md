# Luma Features

A quick reference for all builtin functions, methods, and types supported by Luma.

## Builtin Functions

| Function | Description | Arguments |
|----------|-------------|------------|
| `print(expr)` | Prints expression to stdout | 1 |
| `read()` | Reads line from stdin | 0 |
| `read(n)` | Reads n bytes from stdin | 1 (integer) |
| `run(args...)` | Executes command, returns stdout | Variadic strings OR `list(string)` |
| | *Non-String elements now error instead of silently corrupting args; whitespace is trimmed from cmd and all args; verified working with echo up to 82 elements; clang large list behavior untested after fix* | |
| `file(path)` | Creates file handle | 1 (string) |
| `home()` | Returns `$HOME` path | 0 |
| `time()` | Returns Unix timestamp in ms as int | 0 |
| `json(val)` | Creates JSON handle from string/table | 1 (string or table) |
| `toml(val)` | Creates TOML handle from string/table | 1 (string or table) |
| `string(val)` | Converts value to string | 1 |
| `int(val)` | Converts value to integer | 1 |
| `float(val)` | Converts value to float | 1 |
| `bool(val)` | Converts value to boolean | 1 |
| `tokenize(val)` | Tokenizes string into list of chars | 1 (string) |
| `random(min, max)` | Random int between min and max | 2 (integers) |
| `env(key)` | Gets environment variable | 1 (string) |
| `fetch(url)` | Creates fetch handle | 1 (string) |

## String Methods

| Method | Description | Arguments |
|--------|-------------|------------|
| `len()` | Returns string length | 0 |
| `upper()` | Returns uppercase version | 0 |
| `lower()` | Returns lowercase version | 0 |
| `trim()` | Removes leading/trailing whitespace | 0 |
| `reverse()` | Reverses the string | 0 |
| `chars()` | Returns `list(string)` of characters | 0 |
| `exists()` | Returns `true` if not empty | 0 |
| `contains(x)` | Checks if contains string/char | 1 (string or char) |
| `starts_with(x)` | Checks if starts with string/char | 1 (string or char) |
| `ends_with(x)` | Checks if ends with string/char | 1 (string or char) |
| `repeat(n)` | Repeats string n times | 1 (non-negative integer) |
| `replace(from, to)` | Replaces all occurrences of from with to | 2 (strings) |
| `split(delim)` | Splits on delimiter (default: whitespace) | 0-1 (string or char) |
| `first(char)` | Returns `option(char)` of first char | 0-1 (char type hint) |
| `last(char)` | Returns `option(char)` of last char | 0-1 (char type hint) |
| `as(type)` | Converts to int/float/bool/char/string | 1 (string: "int", "float", etc.) |
| `concat(...)` | Concatenates all arguments onto string (variadic) | Variadic (strings/chars) |
| `hash()` | Returns FNV-1a hash as hex string | 0 |

## List Methods

| Method | Description | Arguments |
|--------|-------------|------------|
| `len()` | Returns number of elements | 0 |
| `exists()` | Returns `true` if not empty | 0 |
| `get(i)` | Gets element at index i | 1 (integer) |
| `add(val)` | Returns new list with val appended | 1 (any) |
| `contains(val)` | Checks if list contains value | 1 (any) |
| `where(val)` | Returns index of value or -1 | 1 (any) |
| `remove(i)` | Removes element at index i | 1 (integer) |
| `reverse()` | Returns reversed list | 0 |
| `sort()` | Returns sorted list (ascending) | 0 |
| `first()` | Returns first element | 0 |
| `last()` | Returns last element | 0 |
| `merge(glue)` | Joins elements into string with separator | 1 (string or char) |

## Table Methods

| Method | Description | Arguments |
|--------|-------------|------------|
| `len()` | Returns number of key-value pairs | 0 |
| `exists()` | Returns `true` if not empty | 0 |
| `has(key)` | Checks if table contains key | 1 (any) |
| `get(key)` | Gets value for key | 1 (any) |
| `set(key, val)` | Sets/updates key-value pair | 2 (key, value) |
| `remove(key)` | Removes key-value pair | 1 (any) |
| `keys()` | Returns `list` of all keys | 0 |
| `values()` | Returns `list` of all values | 0 |

## File Handle Methods

| Method | Description | Arguments |
|--------|-------------|------------|
| `read()` | Reads entire file contents | 0 |
| `write(content)` | Writes content to file (overwrites) | 1 (string) |
| `append(content)` | Appends content to file | 1 (string) |
| `exists()` | Returns `true` if file exists | 0 |
| `age()` | Returns milliseconds since last modification | 0 |
| `list(filter)` | Lists directory contents (filter: "."=files, "/"=dirs, ".ext"=by extension, "subdir/"=contents) | 0-1 (string) |

## JSON Handle Methods

| Method | Description | Arguments |
|--------|-------------|------------|
| `parse()` | Parses JSON string into table | 0 |
| `encode()` | Encodes table into JSON string | 0 |

## TOML Handle Methods

| Method | Description | Arguments |
|--------|-------------|------------|
| `parse()` | Parses TOML string into table | 0 |
| `encode()` | Encodes table into TOML string | 0 |

## Fetch Handle Methods

| Method | Description | Arguments |
|--------|-------------|------------|
| `get()` | Sends GET request, returns response body | 0 |
| `send(body)` | Sends POST request with body, returns response | 0-1 (string) |

## Option Methods

| Method | Description | Arguments |
|--------|-------------|------------|
| `exists()` | Returns `true` if `some(value)` | 0 |
| `or(fallback)` | Returns value or fallback if none | 1 (any) |

## Types Supported

### Primitive Types
- `int` - 64-bit signed integer
- `float` - 64-bit floating point
- `string` - UTF-8 string
- `char` - Unicode character
- `bool` - Boolean (`true`/`false`)
- `void` - No value

### Compound Types
- `list(T)` - Generic list (e.g., `list(int)`, `list(string)`)
- `table(K, V)` - Key-value table (e.g., `table(string, int)`)
- `option(T)` - Optional value (`some(value)` or `none`)

### Special Types
- `struct` - User-defined struct with fields and methods
- `enum` - User-defined enum with variants and optional data

## Known Limitations

- `struct` methods - Must be defined in the same file as struct definition
- `enum` methods - No builtin methods, use defined methods instead
- Large `list(string)` passed to `run()` - Verified working up to 82 elements (echo test); actual clang build failure cause unknown
- `random()` - Uses time-based seed, not cryptographically secure
- `fetch()` - Synchronous only, no async support
- File paths - Must be absolute or relative to current working directory

## Build System Notes

When using `run()` with large lists (simulating clang commands):
- Ensure all files exist before calling `run()`
- Paths are passed as-is; verify they're correct
- `run(args)` where `args` is `list(string)` extracts first element as command, rest as args
- `run("cmd", "arg1", "arg2")` - variadic style also supported
