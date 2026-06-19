; indent inside blocks and braced constructs
[
  (block)
  (declaration_block)
  (field_declaration_list)
  (initializer_list)
  (parameter_list)
  (argument_list)
] @indent

; dedent on closing brackets
[
  "}"
  ")"
  "]"
] @outdent
