; Variables — defined FIRST so all specific patterns below override it
(identifier) @variable

; Keywords — purple
[
  "if" "else" "while" "for" "in" "match"
  "return" "break" "raise" "not"
  "use" "module" "struct"
] @keyword

; print is a keyword token in the grammar, color it as a builtin
"print" @keyword

; Types — light blue
[
  "int" "float" "string" "bool" "char" "void"
  "maybe" "worry" "list" "table"
] @type

; Literals — darkish orange/yellow
(boolean) @boolean
(integer) @number
(float) @number
(char_literal) @string.special

; empty — constant.builtin (lightish yellow)
(empty) @constant.builtin

; Strings — green
(string_literal) @string
(string_content) @string
(escape_sequence) @string.escape

; Interpolation — whole node as string, variable inside as variable.special
(interpolation) @string
(interpolation
  (identifier) @variable.special)

; Comments — muted gray
(comment) @comment

; Functions — defined AFTER variable so they win
(function_declaration
  name: (identifier) @function)

(call_expression
  name: (identifier) @function)

(method_call_expression
  method: (identifier) @function)

(struct_method
  name: (identifier) @function)

; Struct names — @type
(struct_declaration
  name: (identifier) @type)

(struct_instantiation
  name: (identifier) @type)

; Parameters — @variable.parameter
(parameter
  name: (identifier) @variable.parameter)

; Module names
(module_declaration
  name: (identifier) @type)

(use_statement
  module: (identifier) @type)

; Operators — @operator
[
  "+" "-" "*" "/" "%"
  "==" "!=" ">" "<" ">=" "<="
  "&&" "||"
  "=" "+=" "-=" "*=" "/="
] @operator

; Punctuation
[ "(" ")" "{" "}" ] @punctuation.bracket
[ ";" "," "." ":" ] @punctuation.delimiter

(interpolation_close) @string
