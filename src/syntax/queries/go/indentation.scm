; SPDX-License-Identifier: MPL-2.0
; Runyte indentation query for tree-sitter-go 0.25.0.
[(block) (field_declaration_list)] @indent.always
(literal_value) @indent.begin
["{" "(" "["] @indent.opener
[
  (expression_case ":" @indent.opener)
  (default_case ":" @indent.opener)
  (type_case ":" @indent.opener)
  (communication_case ":" @indent.opener)
]
