; SPDX-License-Identifier: MPL-2.0
; Runyte indentation query for tree-sitter-python 0.25.0.
[(function_definition) (class_definition) (if_statement) (for_statement) (while_statement) (with_statement) (try_statement) (match_statement) (case_clause) (elif_clause) (else_clause) (except_clause) (finally_clause)] @indent.always
[(list) (list_comprehension) (dictionary) (dictionary_comprehension) (set) (set_comprehension) (tuple) (argument_list)] @indent.begin
; A compound statement's header ends with `:`. Matching the token rather
; than the statement also covers headers split over several rows and headers
; typed above code they do not yet own, which parse as errors.
":" @indent.opener
