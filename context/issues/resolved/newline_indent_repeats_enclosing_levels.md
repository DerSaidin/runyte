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
`begin_levels + always_levels` was non-zero, and
`DocumentSyntax::newline_indent` counts an `@indent.always` container wherever
it starts, as long as it spans the newline. The row's whitespace already
carries every enclosing level, so each Enter on an ordinary row inside a block
counted the enclosing block a second time and stepped one level deeper. Every
grammar with an `@indent.always` capture was affected, and every indentation
query except TOML's has one; the report named YAML and JSONC. The same applied
to `@indent.tab`: Enter on a Make recipe line added a second tab.

`NewlineIndent` now also carries `opening_levels` and `opening_tab_levels`,
and `edit_newline` adds its one level only from these. They count the begin or
always containers, and the tab contexts, that begin on the newline's own row,
a node captured as both kinds counting once. The existing `begin_levels`,
`always_levels` and `tab_levels` keep their meaning, because they describe the
syntax rather than this frontend's relative policy; the injected-indentation
tests still assert them.

The same double count had been giving a correct level, by accident, below many
openers the queries never captured, as long as some captured block enclosed
them: a YAML key whose value is not typed yet, a Python header typed above code
it does not yet own (which parses as an error) or split over several rows,
Swift `if a {`, Rust `Foo {` and `fn new(`, Go and C `case 1:`, multi-line
calls, JSX in `.js` files, and `else()` in CMake. Counting only containers that
start on the row lost all of these, so the follow-up commit `Open a level
below rows that end with an opening token` added a fourth capture,
`@indent.opener`. A captured node opens a level when it ends exactly where the
row's last token ends; `NewlineIndent::ends_with_opener` reports it apart from
the level counts, because the token usually belongs to a container they already
include, and `edit_newline` adds its level when either says the row opens. `last_row_token` finds that token by stepping back over
trailing whitespace and up to four comments (extras, or the comment enclosing a
comment's own parts), so `{ // note` still ends with its brace, and the query
scans only from that token to the newline, never the whole of a long row. The
queries capture trailing `{`, `(` and `[` in the brace languages and Lua; `:` in
Python and YAML, whose headers and empty keys end with it; the colon of `case`
and `default` labels in Go, C (and so C++), JavaScript (and so TypeScript and
TSX) and Java, and of C++ access specifiers; `=>` of a Rust match arm and `->`
of a Kotlin `when` entry; Bash `then` on its own row; the `>` closing a
multi-line start tag of an HTML element with an end tag; and CMake's flat
`else_command` and `elseif_command`. JavaScript also takes the TSX
`jsx_element` capture, since its grammar parses JSX.

YAML captured `block_mapping` and `block_sequence`, but a mapping begins at its
first key, on a row that also holds content, so `a: 1` as a mapping's first row
would still have indented. It now captures `(block_mapping_pair value:
(block_node))`, which also covers a block scalar after `|`, besides the
trailing `:`. Sequence items are left to the existing list path, which gives a
`- ` row its hanging indent before syntax is consulted. Python, Bash and Lua
capture their clause nodes directly (`elif_clause`, `else_clause`,
`except_clause`, `finally_clause`; `elif_clause`, `else_clause`, `case_item`;
`elseif_statement`, `else_statement`), so a clause row opens a level whichever
rule reaches it.

HTML's `element` capture now excludes void elements with
`#not-any-of?` on the tag name. tree-sitter-html extends `<img src="x">` over
the whitespace after it, so the element appeared to span the next newline and
`<img>` or `<br>` on its own row indented the row below. Requiring an end tag
instead would have stopped `<div>` followed by Enter from indenting before
`</div>` is typed. The owned query compiler now admits captures named with a
leading underscore, the tree-sitter convention for a capture that exists only
for a predicate; the text predicates themselves were already evaluated by
tree-house.

The behavior is covered by
`syntax_newline_inside_a_block_keeps_the_rows_own_indentation`,
`syntax_newline_adds_one_level_below_a_row_that_opens_a_block`, and the table
in `syntax_newline_adds_a_level_exactly_below_rows_that_open_one`, all in
`src/app/tests/editing_and_buffers.rs`, which also holds the corrected
`syntax_newline_multiline_selection_uses_normalized_insertion_row_in_both_directions`:
its old expectation of eight spaces was the bug's four plus four, and it
coincided with the next row's indentation, so the test could not tell the two
rows apart. `newline_indentation_retains_ancestor_captures_in_later_injections`
in `tests/syntax.rs` asserts `opening_levels` beside `always_levels` through
HTML, Markdown and PHP injections, and
`every_canonical_plain_and_owned_capability_query_compiles` in
`src/syntax/mod.rs` compiles every query with the new capture.

Known limitation: a C-family `if`, `for` or `while` whose body is a single
statement on the next row, with no braces, opens nothing, because the row ends
with `)` and an Allman-style brace on the next row would make a level there
wrong. Make's `else` directive is likewise not captured. HTML void elements are
matched by lowercase tag name only. Rows whose opener is none of the tokens
above keep their indentation; they used to indent only because of the bug.

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
