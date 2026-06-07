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

; Import
"use" @keyword

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

(method_receiver
  receiver_name: (identifier) @variable)

(parameter
  name: (identifier) @variable)

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

(module_path
  (identifier) @variable)

; =============================================================================
; Extern declarations
; =============================================================================

(extern_declaration
  abi: (string_literal) @string.special)

(extern_declaration
  name: (identifier) @function)

; =============================================================================
; Test declarations
; =============================================================================

(test_declaration
  name: (string_literal) @string.special)

; =============================================================================
; Compile-time
; =============================================================================

(comptime_if_statement
  "$if" @keyword)

(comptime_or_clause
  "$or" @keyword)

(comptime_expression
  "$" @keyword)

(comptime_expression
  name: (identifier) @function.builtin)

(comptime_field_path
  (identifier) @variable.special)

; =============================================================================
; Assembly
; =============================================================================

(asm_isa_block
  isa: (identifier) @label)

(asm_statement
  (string_literal) @string.special)

(asm_isa_block
  (string_literal) @string.special)

; =============================================================================
; Operators
; =============================================================================

(binary_expression
  operator: _ @operator)

(unary_expression
  operator: _ @operator)

(cast_expression
  "::" @operator)

(assignment_expression
  "=" @operator)

; pointer-type sigil ( *T ) and bitwise-and / readonly-pointer sigil
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
