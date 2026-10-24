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
