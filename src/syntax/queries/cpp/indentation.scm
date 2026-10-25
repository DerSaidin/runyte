; SPDX-License-Identifier: MPL-2.0
; Runyte indentation additions for tree-sitter-cpp 0.23.4.
(declaration_list) @indent.always
; `public:` and its siblings end with a colon that belongs to the class body.
(field_declaration_list ":" @indent.opener)
