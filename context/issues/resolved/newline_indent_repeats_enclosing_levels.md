---
title: "Enter on an ordinary row inside a block added an extra indentation level"
status: resolved
reported: 2026-10-23
resolved: 2026-10-24
commit: a357a62
---

## Resolution

Commit a357a62 (`Stop Enter from re-adding enclosing indentation levels`)
fixed the level `App::edit_newline` adds on top of the row's own leading
whitespace. It added one `editor.indent` level whenever
`begin_levels + always_levels` was non-zero, and `DocumentSyntax::newline_indent`
counts an `@indent.always` container wherever it starts, as long as it spans the
newline. The row's whitespace already carries every enclosing level, so each
Enter on an ordinary row inside a block counted the enclosing block a second
time and stepped one level deeper. Every grammar with an `@indent.always`
capture was affected, and every indentation query except TOML's has one; the
report named YAML and JSONC.
The same applied to `@indent.tab`: Enter on a Make recipe line added a second
tab.

`NewlineIndent` now also carries `opening_levels` and `opening_tab_levels`:
the begin or always containers, and the tab contexts, that begin on the
newline's own row. A node captured as both kinds counts once. `edit_newline`
adds its one level only from these. The existing `begin_levels`,
`always_levels` and `tab_levels` keep their meaning, because they describe the
syntax rather than this frontend's relative policy; the injected-indentation
tests still assert them.

Two query sets relied on the bug. YAML captured `block_mapping` and
`block_sequence`, but a mapping begins at its first key, on a row that also
holds content, so under the corrected rule `a: 1` as a mapping's first row
would still indent. It now captures `(block_mapping_pair value: (block_node))`,
the key whose value is a nested block, including a block scalar after `|`.
Sequence items are left to the existing list path, which already gives a
`- ` row its hanging indent before syntax is consulted. Python, Bash and Lua
indented below `else:`, `elif`, `except`, `finally` and `case` items only
because the enclosing statement spanned the newline; their clause nodes are now
captured directly (`elif_clause`, `else_clause`, `except_clause`,
`finally_clause`; `elif_clause`, `else_clause`, `case_item`;
`elseif_statement`, `else_statement`).

The behavior is covered by
`syntax_newline_inside_a_block_keeps_the_rows_own_indentation` and
`syntax_newline_adds_one_level_below_a_row_that_opens_a_block` in
`src/app/tests/editing_and_buffers.rs`, which also holds the corrected
`syntax_newline_multiline_selection_uses_normalized_insertion_row_in_both_directions`:
its old expectation of eight spaces was the bug's four plus four, and it
coincided with the next row's indentation, so the test could not tell the two
rows apart. `newline_indentation_retains_ancestor_captures_in_later_injections`
in `tests/syntax.rs` asserts `opening_levels` beside `always_levels` through
HTML, Markdown and PHP injections.

Known limitation: CMake `else()` and `elseif()`, and Make `else`, are single
commands or directives in their grammars with no node spanning the branch body,
so nothing opens on their row and Enter below them keeps the row's indentation.
They used to indent only because of the bug.

## Report

Reported as https://github.com/runyte/runyte/issues/23.

In YAML and JSONC, `A` followed by Enter added extra indentation, while `o`
kept the expected indentation. Other languages had not been tried.

Reproduced with default settings, Enter at the end of the second row:

| File | Row | Indentation after Enter | Expected |
| --- | --- | --- | --- |
| YAML `root:\n  a: 1\n  b: 2\n` | `  a: 1` | 6 spaces | 2 |
| JSON/JSONC `{\n  "a": 1,\n  "b": 2\n}\n` | `  "a": 1,` | 6 spaces | 2 |
| Rust `fn f() {\n    let a = 1;\n    let b = 2;\n}\n` | `    let a = 1;` | 8 spaces | 4 |
| Python `def f():\n    a = 1\n    b = 2\n` | `    a = 1` | 8 spaces | 4 |

`o` copies the row's exact leading whitespace and never consults syntax, which
is why the two keys disagreed.
