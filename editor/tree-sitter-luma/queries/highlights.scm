; Keywords — purple
[
  "if" "else" "while" "for" "in" "match"
  "return" "break" "raise" "not"
  "use" "module" "struct"
] @keyword

; Types — light blue
[
  "int" "float" "string" "bool" "char" "void"
  "maybe" "worry" "list" "table"
] @type

; Builtins — light blue (same as types, they're part of the language)
(call_expression
  name: (identifier) @function.builtin
  (#match? @function.builtin "^(print|write|read|input|fetch|file|run|int|float|string|random)$"))

; Literals — darkish orange/yellow
(boolean) @boolean
(integer) @number
(float) @number
(char_literal) @string.special

; empty — lightish yellow
(empty) @constant.builtin

; Strings — green
(string_literal) @string
(string_content) @string
(escape_sequence) @string.escape

; Interpolation — variable inside &{} colored, braces as punctuation
(interpolation) @string
(interpolation
  (identifier) @variable)

; Comments — muted gray
(comment) @comment

; Functions — declarations colored
(function_declaration
  name: (identifier) @function)

; Function calls — colored
(call_expression
  name: (identifier) @function.call)

; Methods — colored
(method_call_expression
  method: (identifier) @function.method)

; Struct names
(struct_declaration
  name: (identifier) @type)

(struct_instantiation
  name: (identifier) @type)

(struct_method
  name: (identifier) @function.method)

; Parameters
(parameter
  name: (identifier) @variable.parameter)

; Module and use
(module_declaration
  name: (identifier) @module)

(use_statement
  module: (identifier) @module)

; Operators — light blue
[
  "+" "-" "*" "/" "%"
  "==" "!=" ">" "<" ">=" "<="
  "&&" "||"
  "=" "+=" "-=" "*=" "/="
] @operator

; Punctuation
[ "(" ")" "{" "}" ] @punctuation.bracket
[ ";" "," "." ":" ] @punctuation.delimiter

; Variables — lowest priority
(identifier) @variable
