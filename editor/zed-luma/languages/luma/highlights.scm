; Keywords
[
  "if" "else" "while" "for" "in" "match"
  "return" "break" "raise" "not"
  "use" "module" "struct"
  "empty"
] @keyword

; Types
[
  "int" "float" "string" "bool" "char" "void"
  "maybe" "worry" "list" "table"
] @type

; Literals
(boolean) @boolean
(integer) @number
(float) @number
(char_literal) @string.special
(empty) @constant.builtin

; Strings — plain content and escape sequences
(string_literal) @string
(string_content) @string
(escape_sequence) @string.escape

; Interpolation — &{var} inside strings
(interpolation
  (identifier) @variable.special)

; Comments
(comment) @comment

; Functions — declarations
(function_declaration
  name: (identifier) @function)

; Functions — calls
(call_expression
  name: (identifier) @function.call)

; Methods
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

; Operators
[
  "+" "-" "*" "/" "%"
  "==" "!=" ">" "<" ">=" "<="
  "&&" "||"
  "=" "+=" "-=" "*=" "/="
] @operator

; Punctuation
[ "(" ")" "{" "}" ] @punctuation.bracket
[ ";" "," "." ":" ] @punctuation.delimiter

; Variables — lowest priority, catches everything else
(identifier) @variable
