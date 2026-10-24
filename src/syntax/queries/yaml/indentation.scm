; SPDX-License-Identifier: MPL-2.0
; Runyte indentation query for tree-sitter-yaml 0.7.2.
; A mapping starts at its first key, on a row that also holds content, so
; capturing it would indent below an ordinary first key. The pair whose value
; is a nested block is the construct that opens a level.
(block_mapping_pair value: (block_node)) @indent.begin
[(flow_mapping) (flow_sequence)] @indent.begin
