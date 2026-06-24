; comments
(comment) @comment

; keywords — import / re-export
"use" @keyword
"fwd" @keyword

; keywords — visibility / linkage
"pub" @keyword
"ext" @keyword

; keywords — type definitions
"def" @keyword
"rec" @keyword
"uni" @keyword

; keywords — storage
"val" @keyword
"var" @keyword

; keywords — functions
"fun" @keyword
"test" @keyword

; keywords — control flow
"if" @keyword
"or" @keyword
"for" @keyword
"ret" @keyword
"brk" @keyword
"cnt" @keyword
"fin" @keyword

; keywords — assembly
"asm" @keyword

; literals
(integer_literal) @number
(float_literal) @number
(char_literal) @string
(string_literal) @string
(nil_literal) @constant

; types
(primitive_type) @type

(type_identifier) @type

(record_declaration
  name: (identifier) @type)

(union_declaration
  name: (identifier) @type)

(type_alias_declaration
  name: (identifier) @type)

(type_parameters
  (identifier) @type)

(generic_type
  name: (identifier) @type)

(generic_type
  name: (type_identifier) @type)

; #[attr] decorators — #[symbol("...")], #[inline], #[align(expr)], #[section], #[library]
(decorator
  "#[" @punctuation.special
  name: (identifier) @attribute)

; functions
(function_declaration
  name: (identifier) @function)

(parameter
  name: (identifier) @variable)

; comptime value parameter marker ( fun f($x: T) )
(parameter
  comptime: "$" @punctuation.special)

; comptime variadic pack parameter ( fun f(va: ...) )
(pack_parameter
  name: (identifier) @variable)

(pack_parameter
  "..." @punctuation.special)

; pack spread ( g(va...) )
(pack_spread_expression
  "..." @punctuation.special)

; comptime field projection ( v.[f] )
(projection_expression
  "." @punctuation.delimiter
  "[" @punctuation.bracket
  "]" @punctuation.bracket)

(call_expression
  function: (identifier) @function)

(call_expression
  function: (field_expression
    field: (identifier) @function.method))

; fields and variables
(field_declaration
  name: (identifier) @property)

; comptime field marker
(field_declaration
  comptime: "$" @punctuation.special)

(field_expression
  field: (identifier) @property)

(initializer_field
  name: (identifier) @property)

(value_declaration
  name: (identifier) @variable)

(variable_declaration
  name: (identifier) @variable)

; modules
(use_declaration
  alias: (identifier) @variable)

(forward_declaration
  alias: (identifier) @variable)

(module_path
  (identifier) @variable)

; test declarations
(test_declaration
  name: (string_literal) @string.special)

; compile-time — declaration-scope $if / $or chain
(comptime_if_declaration
  "$" @keyword
  "if" @keyword)

(comptime_or_declaration_clause
  "$" @keyword
  "or" @keyword)

; compile-time — statement-scope $if / $or chain
(comptime_if_statement
  "$" @keyword
  "if" @keyword)

(comptime_or_clause
  "$" @keyword
  "or" @keyword)

; compile-time — $each unroll over a pack or $fields(T)
(comptime_each_statement
  "$" @keyword
  "each" @keyword
  "in" @keyword)

(comptime_each_statement
  iterator: (identifier) @variable)

(comptime_expression
  "$" @keyword)

(comptime_expression
  name: (identifier) @function.builtin)

(comptime_field_path
  (identifier) @variable.special)

; asm — asm <isa> { raw body }
(asm_statement
  isa: (identifier) @label)

(asm_statement
  body: (asm_body) @string.special)

; operators
(binary_expression
  operator: _ @operator)

(unary_expression
  operator: _ @operator)

; value cast ( :: ) and bit-reinterpret cast ( :~ )
(cast_expression
  operator: _ @operator)

(assignment_expression
  "=" @operator)

; pointer-type sigil ( *T, **T ) and bitwise-and sigil
"*" @operator
"&" @operator

; punctuation
"(" @punctuation.bracket
")" @punctuation.bracket
"{" @punctuation.bracket
"}" @punctuation.bracket
"[" @punctuation.bracket
"]" @punctuation.bracket

";" @punctuation.delimiter
":" @punctuation.delimiter
"," @punctuation.delimiter
"." @punctuation.delimiter
