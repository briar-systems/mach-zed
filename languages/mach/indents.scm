; =============================================================================
; Mach Indent Queries for Zed
; =============================================================================

; Indent each container's body; the closing delimiter (@end) returns the line
; to the opener's level. Scoping @end to the container — rather than a blanket
; @outdent on every closer — avoids spurious dedents on brackets that were
; never indented (if/for/$if conditions, generics, type args, indexing, etc.).
(block "}" @end) @indent
(declaration_block "}" @end) @indent
(field_declaration_list "}" @end) @indent
(initializer_list "}" @end) @indent
(parameter_list ")" @end) @indent
(argument_list ")" @end) @indent
(type_parameters "]" @end) @indent
(type_arguments "]" @end) @indent
