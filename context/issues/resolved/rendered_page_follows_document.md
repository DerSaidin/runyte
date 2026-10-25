---
title: "Opening a Markdown document forgets its rendered page view"
status: resolved
reported: 2026-10-18
resolved: 2026-10-24
commit: ed73778
---

## Resolution

`ed73778` ("Open Markdown documents in the view each pane last showed")
separates opening a document by name from opening a position in its source.
Previously, name-based callers used `App::open_file`, which selected the source
buffer regardless of the rendered page the pane had last shown.

`remembered_markdown_page` derives the choice from `Pane::buffer_history`:
whichever of the source and its page appears most recently is the pane's last
view. No separate preference state is stored, and a split inherits the history
of its parent. `open_document` and `split_document` resume that view for `:open`,
`:split`/`:vsplit` with a path, Finder name rows (including an open document's
`Resource(Buffer)` row), directory-tree and explorer entries, Git changed-file
rows, `gf`, and native launch targets without a position. A `file.md#heading`
link maps the heading into the rendered page when that is the remembered view.
Positional callers retain `open_file`: Finder content matches, language-service
locations, native positional launch targets, and host/`--wait` requests open
source. The buffer list retains separate explicit source and page rows.

`resume_markdown_page` restores jump and destination histories after the
intermediate source visit, so returning from the page goes to the actual
origin of the navigation rather than to the source passed through on the way.
Closing a page removes its history entry and forgets the rendered preference.
`retire_buffer` closes a source's pages with it, excludes those pages from its
fallback choices, and removes their render metadata. Hidden-buffer closing
recognizes a source being read through a visible page and counts all buffers
actually retired. Pages of open documents are exempt from detached
special-buffer retention, so eviction cannot silently erase the view memory.

`markdown_rendered_from` records the source revision used to render each page.
`refresh_markdown_page` regenerates a stale page on arrival through
`switch_buffer`, `jump_in`, or a close fallback, remapping readers through
source offsets. Selecting an already assigned page also refreshes it. An
unchanged source avoids rendering and highlight invalidation. Rendering with
`?` also preserves the positions of other panes reading the shared page.
`open_virtual_page` installs page text before render metadata is attached;
it does not call the arrival refresh path. Refresh itself does not retire
buffers, avoiding retirement re-entrancy.

Regression tests in `src/app/tests/markdown_page_memory.rs` (run with
`cargo test markdown_page_memory`):

- `opening_a_document_by_name_returns_to_the_view_the_pane_left_it_in`.
- `asking_for_the_source_a_pane_already_shows_keeps_it`.
- `requests_for_a_position_in_the_text_open_the_source`.
- `the_finder_opens_an_open_document_by_name_the_way_the_pane_left_it`.
- `the_source_passed_through_is_not_a_place_to_jump_back_to`.
- `a_heading_link_lands_on_the_heading_of_the_rendered_page`.
- `each_pane_remembers_its_own_view_and_a_split_starts_from_its_parent`.
- `explorer_entries_and_the_directory_tree_follow_the_view`.
- `closing_a_document_closes_its_page_and_closing_the_page_forgets_it`.
- `a_page_lives_as_long_as_its_document_despite_special_buffer_retention`.
- `closing_hidden_buffers_keeps_a_document_read_through_its_page`.
- `arriving_at_a_page_from_history_or_the_buffer_list_shows_current_text`.
- `closing_a_buffer_that_falls_back_to_a_page_shows_current_text`.
- `returning_to_an_unchanged_page_does_not_render_or_redraw_it`.
- `closing_hidden_buffers_counts_the_page_that_closes_with_its_document`.
- `selecting_the_page_already_assigned_to_a_pane_refreshes_it`.
- `refreshing_a_shared_page_keeps_both_readers_on_their_text`.
- `retiring_a_document_removes_render_metadata_and_reopening_builds_it_again`.

`space_e_uses_the_markdown_source_directory_and_selects_its_file` in
`src/app/tests/navigation_and_files.rs` also checks that reopening an explorer
entry resumes the rendered view.
`goto_file_from_rendered_markdown_uses_the_source_directory_after_edits` in
`src/app/tests/markdown_positions.rs` checks links before and after arrival
refreshes an edited document's page.

Known limitation: a page shown live in another pane is not re-rendered while
the source is edited; it updates only on arrival or `?`. Host/`--wait` opens
always show the source.

## Report

Rendered Markdown pages are not remembered as the way a document was last
shown. `?` opens a document's rendered page as a separate generated buffer,
and opening the document again by name always shows its source, even when the
page was what the pane last showed for it.

Reproduction:

1. Open `notes.md` and press `?`. The pane shows `[rendered notes.md]`.
2. Open another file, for example with `Space f`, `:open`, or the directory
   tree.
3. Open `notes.md` again the same way. The pane shows the source buffer, not
   the rendered page.

Expected: each document retains its last preview or edit state when switching
buffers. Opening a document by name returns to the view the pane last showed
for it, rendered or source.

The rendered page also stays open after its source document is closed, where
`?` reports `the rendered document is no longer open`, and it can be evicted
by the retention limit for detached special buffers while its document is
still open. A page reached through the buffer list or jump history keeps the
text it was last rendered with, even when the source has changed since.

Reported upstream as GitHub issue #21.
