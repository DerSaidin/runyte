---
title: "Escape after a mid-line append left the caret past the inserted text"
status: resolved
reported: 2026-10-23
resolved: 2026-10-24
commit: e56fffd
---

## Resolution

Commit e56fffd (`Rest the caret on the last appended character after Escape`)
changed `App::normal_mode_selection`, which Escape runs through
`enter_normal_mode`. Under the Runyte grammar it kept each Insert caret where it
was and only clamped it onto the row's characters. After `A`, `o` and `O` the
caret sits at the row end, so the clamp happened to land on the last typed
character; after `a` in mid-line it stayed on the character after the
insertion. With nothing typed, `a` then Escape moved the caret one character
right.

`enter_insert` now records an `AppendInsert` in `App::append_insert` when `a`
opens the session, and `normal_mode_selection` steps each caret back one
grapheme while that record matches the buffer. The step never crosses a row
start, so an append that ended with Enter stays on the new row. `i`, `I` and
`c` leave the record empty and keep the caret on the character the text went
in front of, which already matched Helix; the report's suggestion of Vim's
unconditional step back would have moved those too, and Helix was the stated
reference. The Vim grammar's own step back is unchanged.

A follow-up commit, `Keep the caret where an Insert-mode move put it after
append`, made the record describe the carets rather than only the session.
Keyed by buffer alone, it survived Left, Home, a click, or `Ctrl-w` into
another pane on the same buffer, so Escape still stepped back from wherever the
caret had been put, and where it landed again depended on the entry key. The
record now holds the pane, the buffer's revision, and the selection as the
session's last edit left them. The outer `with_selection_action` boundary
checks it before every action and drops it when the mode, pane, buffer or
selection no longer match, which also covers pointer input that changed the
selection between actions and the workflows that assign `Mode::Normal` or
`Mode::Insert` directly. After an action that changed the buffer's revision it
records the selection the action left, so caret placement an edit makes after
its transaction, such as inside an auto-closed pair, is included. Typing an
existing auto-closer also refreshes the recorded carets: stepping over it
changes no text or revision, but is still part of the append typing.
`enter_normal_mode` and every other Insert entry (`enter_insert(false)`,
`open_line`, `InsertLineEnd`) clear it.

At a line end Runyte's caret lands on the last inserted character after `a`,
`A`, `o` and `O`. Helix rests on the line terminator after `A`, `o` and `O`,
since only its `a` steps back; a Runyte caret never occupies a terminator.

Tests are in `src/app/tests/editing.rs`:
`escape_after_append_rests_on_the_last_inserted_character_like_helix` covers
`i`, `I`, `c`, `a`, `A`, `o` and `O`;
`escape_straight_after_append_returns_to_the_original_character`,
`escape_after_append_steps_back_over_a_whole_grapheme_and_not_across_rows`,
`escape_after_append_steps_back_at_every_caret`,
`an_append_session_left_without_escape_does_not_reach_a_later_insert`, and
`a_caret_move_after_append_keeps_escape_where_the_caret_was_put`.

Known limitation: Runyte leaves a caret where Helix keeps the appended range
selected. Edits applied outside an action, including headless
`apply_transaction` and MCP/context edits, drop the record.

## Report

Reported as https://github.com/runyte/runyte/issues/23.

Where the cursor landed on leaving Insert mode depended on how Insert mode had
been entered. Vim consistently moves back one character after both insert and
append; the report asked for Helix's cursor behavior to be followed
consistently.

Reproduced on `b` in `  abcd`, typing `XY` and pressing Escape:

| Entry | Caret before the fix | Helix | Vim |
| --- | --- | --- | --- |
| `i` | `b`, after the text | `b` | `Y` |
| `a` | `c`, after the text | `Y` | `Y` |
| `A`, `o`, `O` | `Y` | line terminator after `Y` | `Y` |

`a` in mid-line differed from both editors. After `A`, `o` and `O` Runyte
matched Vim and differed from Helix only because a Runyte caret never rests on
a line terminator. The report's two suggestions conflict for `i`, where Helix
and Vim disagree; Helix was followed.
