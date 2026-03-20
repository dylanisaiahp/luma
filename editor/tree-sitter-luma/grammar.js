module.exports = grammar({
  name: "luma",

  extras: ($) => [$.comment, /\s/],

  conflicts: ($) => [
    [$.call_expression, $.struct_instantiation],
    [$.list_expression, $.grouped_expression],
    [$.match_arm, $.match_else],
  ],

  rules: {
    source_file: ($) => repeat($._statement),

    _statement: ($) =>
      choice(
        $.function_declaration,
        $.struct_declaration,
        $.variable_declaration,
        $.return_statement,
        $.print_statement,
        $.if_statement,
        $.while_statement,
        $.for_statement,
        $.match_statement,
        $.break_statement,
        $.raise_statement,
        $.use_statement,
        $.module_declaration,
        $.expression_statement
      ),

    // --- Top level ---

    module_declaration: ($) =>
      seq("module", field("name", $.identifier), ";"),

    use_statement: ($) =>
      seq(
        "use",
        field("module", $.identifier),
        optional(seq(".", choice(
          seq("(", commaSep($.identifier), ")"),
          $.identifier
        ))),
        ";"
      ),

    // --- Functions ---

    function_declaration: ($) =>
      seq(
        field("return_type", $._type),
        field("name", $.identifier),
        "(",
        optional($.parameter_list),
        ")",
        $.block
      ),

    parameter_list: ($) => commaSep1($.parameter),

    parameter: ($) =>
      seq(field("type", $._type), field("name", $.identifier)),

    // --- Structs ---

    struct_declaration: ($) =>
      seq(
        "struct",
        field("name", $.identifier),
        "{",
        repeat(choice($.struct_field, $.struct_method)),
        "}"
      ),

    struct_field: ($) =>
      seq(field("type", $._type), field("name", $.identifier), ";"),

    struct_method: ($) =>
      seq(
        field("return_type", $._type),
        field("name", $.identifier),
        "(",
        optional($.parameter_list),
        ")",
        $.block
      ),

    // --- Statements ---

    variable_declaration: ($) =>
      seq(
        field("type", $._type),
        field("name", $.identifier),
        "=",
        field("value", $._expression),
        optional($.else_error),
        ";"
      ),

    else_error: ($) =>
      seq("else", field("error_var", $.identifier), $.block),

    return_statement: ($) =>
      seq("return", optional($._expression), ";"),

    print_statement: ($) =>
      seq("print", "(", $._expression, ")", ";"),

    break_statement: ($) => seq("break", ";"),

    raise_statement: ($) => seq("raise", $._expression, ";"),

    expression_statement: ($) => seq($._expression, ";"),

    // --- Control flow ---

    if_statement: ($) =>
      prec.right(seq(
        "if",
        field("condition", $._expression),
        field("then", $.block),
        optional($.else_clause)
      )),

    else_clause: ($) =>
      prec.right(seq("else", choice($.if_statement, $.block))),

    while_statement: ($) =>
      seq("while", field("condition", $._expression), $.block),

    for_statement: ($) =>
      choice(
        seq(
          "for",
          "(",
          field("key", $.identifier),
          ",",
          field("val", $.identifier),
          ")",
          "in",
          field("iterable", $._expression),
          $.block
        ),
        seq(
          "for",
          field("var", $.identifier),
          "in",
          "range",
          "(",
          field("start", $._expression),
          ",",
          field("end", $._expression),
          ")",
          $.block
        ),
        seq(
          "for",
          field("var", $.identifier),
          "in",
          field("iterable", $._expression),
          $.block
        )
      ),

    // --- Match ---
    // Match arms use a single statement followed by optional block,
    // or just a block — this avoids the else ambiguity entirely.

    match_statement: ($) =>
      seq(
        "match",
        field("value", $._expression),
        "{",
        repeat($.match_arm),
        optional($.match_else),
        "}"
      ),

    // Each arm: pattern: statement  (one statement only — no if/else ambiguity)
    match_arm: ($) =>
      prec.right(seq(
        field("pattern", $._match_pattern),
        ":",
        field("body", $._match_body)
      )),

    match_else: ($) =>
      prec.right(seq(
        "else",
        ":",
        field("body", $._match_body)
      )),

    // A match body is either a block or a single non-if statement
    // Using a block removes the else ambiguity entirely
    _match_body: ($) =>
      choice(
        $.block,
        $.return_statement,
        $.print_statement,
        $.break_statement,
        $.raise_statement,
        $.expression_statement,
        $.variable_declaration,
        $.while_statement,
        $.for_statement,
        $.match_statement
      ),

    _match_pattern: ($) =>
      choice(
        $.integer,
        $.string_literal,
        $.range_pattern,
        $.set_pattern
      ),

    range_pattern: ($) =>
      seq("range", "(", $.integer, ",", $.integer, ")"),

    set_pattern: ($) =>
      prec(1, seq("(", commaSep1(choice($.string_literal, $.integer)), ")")),

    // --- Block ---

    block: ($) => seq("{", repeat($._statement), "}"),

    // --- Expressions ---

    _expression: ($) =>
      choice(
        $.binary_expression,
        $.unary_expression,
        $.call_expression,
        $.method_call_expression,
        $.field_access_expression,
        $.struct_instantiation,
        $.assignment_expression,
        $.identifier,
        $.integer,
        $.float,
        $.string_literal,
        $.char_literal,
        $.boolean,
        $.empty,
        $.list_expression,
        $.table_expression,
        $.grouped_expression
      ),

    grouped_expression: ($) =>
      seq("(", $._expression, ")"),

    assignment_expression: ($) =>
      prec.right(1, seq(
        field("name", $.identifier),
        field("op", choice("=", "+=", "-=", "*=", "/=")),
        field("value", $._expression)
      )),

    binary_expression: ($) =>
      choice(
        prec.left(5,  seq($._expression, "||", $._expression)),
        prec.left(6,  seq($._expression, "&&", $._expression)),
        prec.left(10, seq($._expression, choice("==", "!="), $._expression)),
        prec.left(20, seq($._expression, choice(">", "<", ">=", "<="), $._expression)),
        prec.left(30, seq($._expression, choice("+", "-"), $._expression)),
        prec.left(40, seq($._expression, choice("*", "/", "%"), $._expression))
      ),

    unary_expression: ($) =>
      choice(
        prec(50, seq("not", $._expression)),
        prec(50, seq("-", $._expression))
      ),

    call_expression: ($) =>
      prec(60, seq(
        field("name", $.identifier),
        "(",
        optional(commaSep($._expression)),
        ")"
      )),

    method_call_expression: ($) =>
      prec.left(60, seq(
        field("object", $._expression),
        ".",
        field("method", $.identifier),
        "(",
        optional(commaSep($._expression)),
        ")"
      )),

    field_access_expression: ($) =>
      prec.left(59, seq(
        field("object", $._expression),
        ".",
        field("field", $.identifier)
      )),

    struct_instantiation: ($) =>
      prec(61, seq(
        field("name", $.identifier),
        "(",
        commaSep1(seq($.identifier, ":", $._expression)),
        ")"
      )),

    list_expression: ($) =>
      prec(1, seq("(", commaSep1($._expression), optional(","), ")")),

    table_expression: ($) =>
      prec(2, seq(
        "(",
        commaSep1(seq($._expression, ":", $._expression)),
        ")"
      )),

    // --- Strings ---

    string_literal: ($) =>
      seq(
        '"',
        repeat(choice(
          $.string_content,
          $.interpolation,
          $.escape_sequence
        )),
        '"'
      ),

    string_content: ($) => token.immediate(/[^"\\&]+/),

    escape_sequence: ($) => token.immediate(/\\[ntr\\"]/),

    interpolation: ($) =>
      seq(
        token.immediate("&{"),
        field("variable", $.identifier),
        "}"
      ),

    // --- Char ---

    char_literal: ($) =>
      seq("'", token.immediate(/[^'\\]/), "'"),

    // --- Types ---

    _type: ($) =>
      choice(
        $.primitive_type,
        $.maybe_type,
        $.worry_type,
        $.list_type,
        $.table_type,
        $.identifier
      ),

    primitive_type: ($) =>
      choice("int", "float", "string", "bool", "char", "void"),

    maybe_type: ($) => seq("maybe", "(", $._inner_type, ")"),
    worry_type: ($) => seq("worry", "(", $._inner_type, ")"),
    list_type:  ($) => seq("list",  "(", $._inner_type, ")"),

    table_type: ($) =>
      seq("table", "(", $._inner_type, ",", $._inner_type, ")"),

    _inner_type: ($) =>
      choice("int", "float", "string", "bool", "char"),

    // --- Literals ---

    integer: ($) => /\d+/,
    float:   ($) => /\d+\.\d+/,
    boolean: ($) => choice("true", "false"),
    empty:   ($) => "empty",

    // --- Identifiers and comments ---

    identifier: ($) => /[a-zA-Z_][a-zA-Z0-9_]*/,

    comment: ($) => token(seq("#", /.*/)),
  },
});

function commaSep(rule) {
  return optional(commaSep1(rule));
}

function commaSep1(rule) {
  return seq(rule, repeat(seq(",", rule)));
}
