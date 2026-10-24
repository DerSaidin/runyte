; SPDX-License-Identifier: MPL-2.0
; Runyte indentation query for tree-sitter-html 0.23.2.
; A void element has no end tag, and the grammar extends it over the
; whitespace that follows, so it would appear to span the newline after it.
((element (start_tag (tag_name) @_void)) @indent.always
  (#not-any-of? @_void
    "area" "base" "br" "col" "embed" "hr" "img" "input" "link" "meta" "source" "track" "wbr"))
[(script_element) (style_element)] @indent.always
; The row closing a start tag split over several rows, for elements that
; have an end tag; void elements open nothing.
(element (start_tag ">" @indent.opener) (end_tag))
