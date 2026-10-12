# Separate soft-wrap setting for rendered Markdown

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
