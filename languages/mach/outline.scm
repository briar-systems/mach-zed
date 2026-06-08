; =============================================================================
; Mach Code Outline Queries for Zed
; =============================================================================

; Functions
(function_declaration
  name: (identifier) @name) @item

; Records
(record_declaration
  name: (identifier) @name) @item

; Unions
(union_declaration
  name: (identifier) @name) @item

; Type aliases
(type_alias_declaration
  name: (identifier) @name) @item

; Value declarations
(value_declaration
  name: (identifier) @name) @item

; Variable declarations
(variable_declaration
  name: (identifier) @name) @item

; Test declarations
(test_declaration
  name: (string_literal) @name) @item

; Forward re-exports
(forward_declaration
  path: (module_path) @name) @item
