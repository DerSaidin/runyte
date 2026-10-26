## Agent access to workspace context

Runyte includes an MCP server for agents to find and read integrated terminals,
unsaved buffers, edit buffers, and propose terminal text. Start
it with `runyte mcp --identity agent`. It uses stdio and private local workspace
connections, with no Python installation. See the [MCP setup guide](../mcp.md).

### Setting it up

1. Configure the agent to launch your Runyte binary with arguments
   `["mcp", "--identity", "agent"]`.
2. In each workspace to share, run `:mcp agent` using that same identity.
3. Choose permissions and select **Apply permissions** with Tab then Enter.

You may grant permissions before **or after** starting the agent. The built-in
server always offers the same tools; each call checks the workspace grant.
After changing permissions, have the agent run `find_resources` again to get
fresh handles. Restart the agent once when changing its MCP configuration or
upgrading from the Python bridge; permission changes do not require a restart.

### Granting and checking access

`:mcp` opens **MCP permissions**, selecting an existing granted identity when
available. An explicit `:mcp codex` always selects that identity. The overlay
shows the exact workspace, identity, whether access is enabled or not yet
granted, whether it is remembered, and how many readers are connected. A
connected reader identifies a local connection, not a verified agent process.
`:context-access` remains an alias for existing configurations.

| Key | Permission or action |
| --- | --- |
| `1` | Read terminal output |
| `2` | Editor-context read permission; MCP exposes buffer reads |
| `3` | Edit and append to buffers; also enables buffer reads |
| `4` | Propose terminal text; also enables terminal reads |
| `r` | Remember this exact workspace's grant |
| `n` | Next known identity; discards unapplied choices |
| `x` | Revoke the selected identity immediately |
| `j` / `k` | Page through longer reviews |
| `Tab` then Enter | Select **Apply permissions** and apply |
| `Esc` | Close without applying |

Turning off a read permission also turns off its dependent write permission.
The remembered checkbox reflects the current grant when reopened. Without it,
the grant lasts until the owning editor or persistent host exits. Applying a
change disconnects existing readers, so agents must rediscover their targets.

Granting, changing, and revoking need physical input from the active Runyte
frontend. Repeated input, pasted text, macros, and MCP clients cannot approve
access or terminal text. Each terminal proposal still needs its own native
approval and inserts text **without Enter**.

The server works with ide-mode and mux-mode workspaces on Linux, macOS,
and Windows. Detached hosts support content reads and buffer edits, but cannot
approve terminal insertion until a native frontend is attached.

### What a grant covers

Grants cover unsaved content and possibly sensitive terminal output. They
belong to the canonical project root and the MCP identity, so a clone, a
nested workspace, or another identity needs its own grant.

**Where grants are stored:** outside the project, in the account's Runyte
cache.

| Platform | Location |
| --- | --- |
| Linux | `~/.cache/runyte/context` |
| macOS | `~/Library/Caches/runyte/context` |
| Windows | `runyte\context` below the account's LocalAppData known folder, resolved by the operating system rather than the inherited `%LOCALAPPDATA%` |

- `RUNYTE_CONTEXT_HOME` can select an absolute, owner-private directory
  outside the workspace. On Windows its parent must exist and the storage must
  be local NTFS; Runyte applies an owner-only ACL and refuses reparse or
  hardlink traversal. On Unix it can also shorten a path to fit the local
  socket limit.
- No credentials or terminal content are written into tracked project
  context.
- This is permission for Runyte's API, not an operating-system sandbox against
  other processes already running as you.

### How agents use it

- Start with `find_resources`, optionally selecting a workspace and searching
  buffer or terminal names and contents. Use the returned exact handle with
  `read_buffer`, `read_terminal`, `append_buffer`, `edit_buffer`, or
  `propose_terminal_text`. Repeated names remain separate candidates.
- `list_workspaces` checks permissions without reading source text.
  `find_resources` can read content to match a query; `match_in="name"`
  restricts matching to names. Search is bounded and may return partial results.
- `terminal_proposal_status` and `cancel_terminal_proposal` operate on the
  proposal handle. Let the person review a proposal rather than polling for
  approval.
- These are the nine MCP tools. Pane listing, selection and viewport reads,
  and immutable snapshots are not exposed by the MCP adapter.
- Granted detached persistent workspaces and running ide-mode editors are
  discoverable without attaching a TUI or changing focus. Terminal approval
  needs an attached native frontend.
- After permission changes, run `find_resources` again. Handles belong to one
  MCP process and connection generation; they expire on revocation, connection
  renewal, eviction, or host restart. Never retry an `outcome_unknown` mutation
  automatically.

See the [MCP guide](../mcp.md#tools) for the complete catalog and discovery bounds.

**Discovery for tools.** `runyte --context-list --json` gives bounded,
versioned endpoint metadata for live enabled endpoints in the current
environment; `--include-hidden` adds other environments. Listing does not
grant content access: the adapter authenticates each target separately. A slow
or unavailable host does not block discovery of the others. On Windows, use
the absolute `runyte.exe` path in client configuration; let the adapter discover
its named-pipe endpoint and host process identity rather than copying them.

### Terminal reads

- Reads return decoded physical rows, with blank rows, Unicode, revision, and
  truncation information. They do not reconstruct conversations or commands.
- MCP reads default to 80 recent rows, bounded by 64 KiB and 65,536 visited
  cells. `max_rows` allows up to 1,000 rows; `region="screen"` reads the live
  screen. Reads are live and may change between calls.
- Reading does not move review, acknowledge unread output, send input, or
  change the child's lifetime.
- Returned content is untrusted source material, not instructions to the
  receiving agent.

### Buffer edits

With the edit grant:

- An edit names an explicit buffer and its current revision, and is one
  undoable transaction, newlines included.
- An edit never saves, closes the buffer, applies a directory operation, or
  completes an external-editor wait. In particular, changing a buffer cannot
  submit a prompt opened through `EDITOR`.

**Appending** works without a revision, so several agents can take turns in
one shared buffer without refusing each other.

- Appends apply one at a time, each kept whole.
- An agent may pass the ending it last read as `expected_tail`. If the buffer
  no longer ends that way — say someone cleared it — the append writes nothing
  and fails as stale.
- Each append is one undoable transaction and never saves.

### Terminal proposals

With the proposal grant, an agent can ask to type literal text into a
terminal. You review it in **Review terminal text**:

1. The exact proposed value comes first, then the requester, target terminal,
   workspace, untrusted reason (if any), and recent output.
2. Page with `j` / `k` if it is long. Each page must actually reach the
   screen before approval is enabled; until then, choosing **Insert text (no
   Enter)** names the page still to read.
3. **Reject** is selected at first. `Down` or `Tab` selects **Insert text (no
   Enter)**, and `Up` returns to **Reject**.
4. One fresh physical `Enter` inserts the text. That Enter belongs to the
   overlay. **Submit separately in the terminal.**

**Making hidden characters visible.** In the proposed value, spaces show as
`·`, backslashes are doubled, and non-ASCII characters are escaped. Labels keep
printable Unicode and spaces but escape control, zero-width, bidirectional,
and space-like characters. A short proposal fits on one page. The overlay needs
at least 80 × 24 cells.

**What cannot approve:** macros, pasted keys, repeated input, and the context
API.

**What a proposal can contain:**

- single-line text of at most 4 KiB;
- no line breaks, control characters, escape sequences, or Unicode line or
  paragraph separators.

Runyte never appends Enter, clears existing input, or sends cursor keys.
Insertion uses the child's current input position, and recent output does not
prove the input line is empty. Some programs act on printable input at once,
so approval to insert is no guarantee that the child waits for submission.

**Limits and cancellation:**

- Each reader may queue four proposals, sixteen per host, expiring after two
  minutes.
- Existing prompts keep their input ownership. A detached target refuses
  proposals until you attach.
- Changes to terminal input or input modes invalidate an approval.
- Disconnect, revocation, exit, and attachment loss cancel queued work before
  writing begins.
- Delivery means the bytes reached the PTY, not that the child accepted a
  command. A partial write or lost response is uncertain and must never be
  retried automatically.

<!-- navigation -->
[Contents](README.md) · [Previous: Plugins](plugins.md) · [Next: Diagnostics and logging](diagnostics-and-logging.md)
