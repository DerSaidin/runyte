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

`enter_insert` now records the buffer in `App::append_insert` when `a` opens
the session, and `normal_mode_selection` steps each caret back one grapheme
when that record matches. The step never crosses a row start, so an append
that ended with Enter stays on the new row rather than returning to the line
terminator Helix would rest on. `i`, `I` and `c` leave the record empty and
keep the caret on the character the text went in front of, which already
matched Helix; the report's suggestion of Vim's unconditional step back would
have moved those too, and Helix was the stated reference. The Vim grammar's
own step back is unchanged.

The record is cleared by `enter_normal_mode`, by every other Insert entry
(`enter_insert(false)`, `open_line`, `InsertLineEnd`), and at the outer
`with_selection_action` boundary whenever an action begins outside Insert
mode. Several workflows assign `Mode::Normal` or `Mode::Insert` directly;
the boundary check means a session they end cannot carry its record into an
Insert session they start later.

Runyte still leaves a caret where Helix keeps the appended range selected.

Tests are in `src/app/tests/editing.rs`:
`escape_after_append_rests_on_the_last_inserted_character_like_helix` covers
`i`, `I`, `c`, `a`, `A`, `o` and `O`;
`escape_straight_after_append_returns_to_the_original_character`,
`escape_after_append_steps_back_over_a_whole_grapheme_and_not_across_rows`,
`escape_after_append_steps_back_at_every_caret`, and
`an_append_session_left_without_escape_does_not_reach_a_later_insert`.

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
| `A`, `o`, `O` | `Y` | `Y` | `Y` |

Only `a` in mid-line differed from both editors. The report's two suggestions
conflict for `i`, where Helix and Vim disagree; Helix was followed.
