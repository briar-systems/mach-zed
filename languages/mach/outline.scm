; functions
(function_declaration
  name: (identifier) @name) @item

; records
(record_declaration
  name: (identifier) @name) @item

; unions
(union_declaration
  name: (identifier) @name) @item

; type aliases
(type_alias_declaration
  name: (identifier) @name) @item

; value declarations
(value_declaration
  name: (identifier) @name) @item

; variable declarations
(variable_declaration
  name: (identifier) @name) @item

; test declarations
(test_declaration
  name: (string_literal) @name) @item

; forward re-exports
(forward_declaration
  path: (module_path) @name) @item
