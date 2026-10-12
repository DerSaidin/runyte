---
title: "Rendered Markdown and code share the same soft-wrap setting"
status: resolved
reported: 2026-10-10
resolved: 2026-10-12
commit: c0ebf0e
---

## Resolution

Commit `c0ebf0e` (`Wrap rendered Markdown independently of the general soft-wrap toggle`)
fixes the coupling in `App::pane_soft_wrap`. Previously this function used only
`editor.soft_wrap`, so a rendered page could wrap only when ordinary source
buffers also wrapped. It now recognizes generated Markdown buffer identity and
applies `editor.always_wrap_rendered_markdown`, defaulting to true, before the
existing comparison and line-length viability restrictions.

False means follow the general setting rather than prohibit wrapping. Markdown
source and other generated pages retain their existing policy. The settings
registry exposes the new boolean with immediate preview, rollback and YAML
persistence. `Space p s` still changes the general setting, and the status-line
arrow still reports it. Toggle feedback explains when the rendered page remains
wrapped; it does not claim this while comparison restrictions prevent wrapping.
The Zen documentation describes this effective policy too.

Regression tests in `src/app/tests/markdown_tables.rs`:

- `rendered_markdown_wrap_override_keeps_general_toggle_and_status_independent`
  covers every setting combination, scratch rendering, visual rows, transitions
  back to source, global toggling, feedback and status snapshots.
- `rendered_markdown_wrap_setting_previews_rolls_back_and_persists` checks missing
  configuration defaults, settings preview/cancel/save, reload and preserved
  comments in temporary configuration storage.
- `markdown_file_source_stays_unwrapped_while_its_page_wraps` covers file-backed
  Markdown independently of scratch rendering.
- `rendered_markdown_override_does_not_wrap_comparison_panes` covers the comparison
  exception, accurate feedback and restoration of wrapping after comparison.
- The existing `markdown_tables_*` tests now exercise the default override with
  general soft wrap disabled, including cell layout, movement, hit testing,
  selections, search, yank, resizing, split widths and horizontal overflow.

## Report

Reported in https://github.com/runyte/runyte/issues/17.

Rendered Markdown and code buffers both follow `editor.soft_wrap`. Enabling
wrap for Markdown also wraps code. Rendered Markdown should wrap to the pane
width while code remains unwrapped. Discussion on the issue supports wrapping
rendered Markdown by default to improve readability.

Open a Markdown file, render it with `?`, then toggle `Space p s`. Returning to
source or opening code shows that the same toggle affects both views.

The agreed setting is `editor.always_wrap_rendered_markdown`, defaulting to
`true`. False makes rendered pages follow the general soft-wrap toggle. The
status-line arrow continues to show the general setting. Markdown source is
unaffected, and comparison-pane and large-line wrapping restrictions remain.
