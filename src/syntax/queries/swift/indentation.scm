; SPDX-License-Identifier: MPL-2.0
; Runyte indentation query for tree-sitter-swift 0.7.3.
[(protocol_body) (class_body) (enum_class_body) (function_body) (computed_property) (computed_getter) (computed_setter) (switch_entry)] @indent.always
[(tuple_expression) (array_literal) (dictionary_literal) (lambda_literal)] @indent.begin
; A switch brace is left out: Swift style keeps `case` at the switch's
; own level.
[
  (if_statement "{" @indent.opener)
  (guard_statement "{" @indent.opener)
  (for_statement "{" @indent.opener)
  (while_statement "{" @indent.opener)
  (repeat_while_statement "{" @indent.opener)
  (do_statement "{" @indent.opener)
  (catch_block "{" @indent.opener)
  (function_body "{" @indent.opener)
  (class_body "{" @indent.opener)
  (protocol_body "{" @indent.opener)
  (enum_class_body "{" @indent.opener)
  (computed_property "{" @indent.opener)
  (computed_getter "{" @indent.opener)
  (computed_setter "{" @indent.opener)
  (willset_didset_block "{" @indent.opener)
  (lambda_literal "{" @indent.opener)
]
["(" "["] @indent.opener
