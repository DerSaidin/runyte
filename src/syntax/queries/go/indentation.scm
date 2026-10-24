; SPDX-License-Identifier: MPL-2.0
; Runyte indentation query for tree-sitter-go 0.25.0.
[(block) (field_declaration_list)] @indent.always
(literal_value) @indent.begin
; A switch or select brace is left out: gofmt keeps `case` at the
; statement's own level.
[
  (block "{" @indent.opener)
  (literal_value "{" @indent.opener)
  (field_declaration_list "{" @indent.opener)
  (interface_type "{" @indent.opener)
]
["(" "["] @indent.opener
[
  (expression_case ":" @indent.opener)
  (default_case ":" @indent.opener)
  (type_case ":" @indent.opener)
  (communication_case ":" @indent.opener)
]
