; =============================================================================
; Mach Highlight Queries for Zed
; =============================================================================

; =============================================================================
; Comments
; =============================================================================

(comment) @comment

; =============================================================================
; Keywords
; =============================================================================

; Import / re-export
"use" @keyword
"fwd" @keyword

; Visibility / linkage
"pub" @keyword
"ext" @keyword

; Type definition keywords
"def" @keyword
"rec" @keyword
"uni" @keyword

; Storage keywords
"val" @keyword
"var" @keyword

; Function keywords
"fun" @keyword
"test" @keyword

; Control flow
"if" @keyword
"or" @keyword
"for" @keyword
"ret" @keyword
"brk" @keyword
"cnt" @keyword
"fin" @keyword

; Assembly
"asm" @keyword

; =============================================================================
; Literals
; =============================================================================

(integer_literal) @number
(float_literal) @number
(char_literal) @string
(string_literal) @string
(nil_literal) @constant
(varargs_expression) @punctuation.special

; =============================================================================
; Types
; =============================================================================

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

; =============================================================================
; Functions
; =============================================================================

(function_declaration
  name: (identifier) @function)

(parameter
  name: (identifier) @variable)

; comptime value parameter marker ( fun f($x: T) )
(parameter
  comptime: "$" @punctuation.special)

(call_expression
  function: (identifier) @function)

(call_expression
  function: (field_expression
    field: (identifier) @function.method))

; =============================================================================
; Fields and variables
; =============================================================================

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

; =============================================================================
; Modules
; =============================================================================

(use_declaration
  alias: (identifier) @variable)

(forward_declaration
  alias: (identifier) @variable)

(module_path
  (identifier) @variable)

; =============================================================================
; Test declarations
; =============================================================================

(test_declaration
  name: (string_literal) @string.special)

; =============================================================================
; Compile-time
; =============================================================================

; declaration-scope $if / $or chain
(comptime_if_declaration
  "$" @keyword
  "if" @keyword)

(comptime_or_declaration_clause
  "$" @keyword
  "or" @keyword)

; statement-scope $if / $or chain
(comptime_if_statement
  "$" @keyword
  "if" @keyword)

(comptime_or_clause
  "$" @keyword
  "or" @keyword)

(comptime_expression
  "$" @keyword)

(comptime_expression
  name: (identifier) @function.builtin)

(comptime_field_path
  (identifier) @variable.special)

; =============================================================================
; Assembly
; =============================================================================

; asm <isa> { raw body }
(asm_statement
  isa: (identifier) @label)

(asm_statement
  body: (asm_body) @string.special)

; =============================================================================
; Operators
; =============================================================================

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

; =============================================================================
; Punctuation
; =============================================================================

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
