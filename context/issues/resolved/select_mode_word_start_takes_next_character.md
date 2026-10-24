---
title: "Select-mode w selected the first character of the next word"
status: resolved
reported: 2026-10-18
resolved: 2026-10-24
commit: 801bd93
---

## Resolution

Commit 801bd93 (`Stop Select-mode w from taking the next word's first
character`) changed `App::word_motion` in `src/app/editing.rs`. In Normal mode
with `editor.selecting_motions` on, it already ran `select_word_motion`, which
walks Helix's half-open word span and converts the result back to Runyte's
inclusive ranges. In Select mode it fell through to `motion_with_extension`,
which moved the head to the offset a caret would move to and extended there.
For `w` and `W` that offset is the first character of the next word, and an
inclusive range extended onto it selects it. `e`, `b` and the find motions
already land on the last character they take, so only the forward word-start
motions selected one character too many.

Select mode now runs the same `select_word_motion` walk for all six word
motions and keeps the existing anchor, taking only the head of the range the
walk leaves. This is Helix's `extend_word_impl`, which computes the word range
a fresh motion would select and puts the cursor at its end. It applies whether
or not `editor.selecting_motions` is on, since Select mode extends either way.
`v w w` now selects `one two `, and a backward selection keeps its anchor as
`w` turns the head round.

The walk reads inclusive ranges. A syntax expansion (`Space x e`) leaves a
half-open range in Select mode, and walking from its exclusive end would start
one character late: with `one` expanded in `one + two`, `w` would have selected
`one + `. A selection whose semantics are not `Runyte` is therefore converted
with `vim_half_open_to_inclusive` before the walk, and the result is marked
`Runyte`. As a side effect `e` and `b` after a syntax expansion, which were
already one character off, now match Helix too.

The Vim grammar is unchanged. Its visual `w` reaches `motion_with_extension`
through `vim_motion_kind` and still takes the next word's first character, as
Vim's does.

Tests are in `src/app/tests/editing.rs`:
`select_mode_word_motions_extend_to_the_selecting_head` covers `v w`, `v w w`,
`v 2 w`, `v W`, `v e`, `v w b`, the document end, a row break and a backward
selection, with `editor.selecting_motions` on and off;
`select_mode_word_motions_extend_a_half_open_syntax_selection` covers `w`, `e`
and `b` after a syntax expansion; and
`selecting_motions_leave_other_modes_and_keys_as_they_were` now expects
`v w w` to stop at the space after `two`.
`visual_yank_includes_the_character_under_the_cursor` in
`src/app/tests/search_and_pickers.rs` expects `Test ` from `v w y` under the
Runyte grammar and still `Test T` under the Vim grammar.

Known limitation: terminal review dispatches its own review motions before
`word_motion` is reached, so its Select-mode `w` is not changed by this fix.

## Report

Reported as https://github.com/runyte/runyte/issues/20.

With `one two three` and the cursor on the first `o`, pressing `v w` selected
`one t` instead of `one ` (with the trailing space). Normal-mode `w` selected
`one ` as expected.

Reproduced with the default configuration, yanking after each gesture:

| Keys | Range (anchor, head) | Selected |
| --- | --- | --- |
| `w` (Normal mode) | (0, 3) | `one ` |
| `v w` | (0, 4) | `one t` |
| `v w w` | (0, 8) | `one two t` |
| `v W` | (0, 4) | `one t` |
| `v e` | (0, 2) | `one` |

Helix selects `one ` for `v w`.
