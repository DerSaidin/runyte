// SPDX-License-Identifier: MPL-2.0
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use serde::Deserialize;
use std::{fmt::Write, str::FromStr, sync::OnceLock};
use syntect::{
    highlighting::{Color, ScopeSelectors, StyleModifier, Theme, ThemeItem, ThemeSet},
    parsing::SyntaxSet,
};

/// The editor theme's colours for a generated page, as sRGB channels.
///
/// Channels rather than CSS text: every colour reaches the stylesheet through
/// [`css`], so nothing in a request can add a declaration of its own.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Palette {
    pub dark: bool,
    background: [u8; 3],
    foreground: [u8; 3],
    muted: [u8; 3],
    surface: [u8; 3],
    code: [u8; 3],
    border: [u8; 3],
    link: [u8; 3],
    heading: [u8; 3],
    error: [u8; 3],
    #[serde(default)]
    pub syntax: Vec<(String, [u8; 3])>,
}

/// Runyte's syntax scope names and the TextMate selectors Syntect's grammars
/// use for the same things. Where a grammar nests punctuation inside a
/// string, comment or heading, the two-part selector keeps the delimiters in
/// the enclosing colour, as Runyte's own highlighting does.
const SCOPE_SELECTORS: &[(&str, &str)] = &[
    ("attribute", "entity.other.attribute-name"),
    ("comment", "comment, comment punctuation"),
    (
        "constant",
        "constant, support.constant, variable.other.constant",
    ),
    (
        "constructor",
        "entity.name.function.constructor, support.function.constructor",
    ),
    (
        "function",
        "entity.name.function, support.function, variable.function",
    ),
    ("keyword", "keyword, storage"),
    ("label", "entity.name.label"),
    ("markup.bold", "markup.bold"),
    (
        "markup.heading",
        "markup.heading, markup.heading punctuation, entity.name.section",
    ),
    ("markup.italic", "markup.italic"),
    ("markup.link.text", "string.other.link"),
    ("markup.link.url", "markup.underline.link"),
    (
        "markup.list",
        "markup.list punctuation.definition.list_item",
    ),
    ("markup.quote", "markup.quote"),
    ("markup.raw", "markup.raw"),
    ("markup.strikethrough", "markup.strikethrough"),
    (
        "namespace",
        "entity.name.namespace, entity.name.module, support.other.namespace",
    ),
    ("number", "constant.numeric"),
    ("operator", "keyword.operator"),
    (
        "property",
        "variable.other.member, variable.other.property, support.type.property-name, \
         meta.mapping.key string, meta.mapping.key string punctuation, \
         entity.name.tag.yaml, meta.object-literal.key",
    ),
    ("punctuation", "punctuation"),
    ("string", "string, string punctuation"),
    ("tag", "entity.name.tag"),
    (
        "type",
        "entity.name.type, entity.name.class, entity.name.struct, entity.name.enum, \
         entity.name.trait, entity.name.interface, support.type, support.class",
    ),
    ("variable", "variable"),
];
/// Bounds the scope list a request may carry; Runyte names fewer than this.
pub const MAX_SYNTAX_SCOPES: usize = 64;

fn css([red, green, blue]: [u8; 3]) -> String {
    format!("#{red:02x}{green:02x}{blue:02x}")
}
fn syntect_color([r, g, b]: [u8; 3]) -> Color {
    Color { r, g, b, a: 0xff }
}
/// A Syntect theme painting Runyte's scope colours on the palette's code ground.
fn syntect_theme(palette: &Palette) -> Theme {
    let mut theme = Theme::default();
    theme.settings.foreground = Some(syntect_color(palette.foreground));
    theme.settings.background = Some(syntect_color(palette.surface));
    theme.scopes = palette
        .syntax
        .iter()
        .filter_map(|(name, colour)| {
            let (_, selectors) = SCOPE_SELECTORS.iter().find(|(scope, _)| scope == name)?;
            Some(ThemeItem {
                scope: ScopeSelectors::from_str(selectors).ok()?,
                style: StyleModifier {
                    foreground: Some(syntect_color(*colour)),
                    ..Default::default()
                },
            })
        })
        .collect();
    theme
}
/// Colour overrides layered over `preview.css`, which keeps the layout.
fn themed_stylesheet(palette: &Palette) -> String {
    let mut sheet = String::new();
    let _ = write!(
        sheet,
        "html {{ background: {background}; color: {foreground}; }}\n\
         h1, h2, h3, h4, h5, h6 {{ color: {heading}; }}\n\
         h1, h2 {{ border-bottom-color: {border}; }}\n\
         a {{ color: {link}; }}\n\
         blockquote {{ color: {muted}; border-left-color: {border}; }}\n\
         pre {{ background: {surface}; }}\n\
         code {{ background: {code}; }}\n\
         pre code {{ background: transparent; }}\n\
         th, td {{ border-color: {border}; }}\n\
         tr:nth-child(even) {{ background: {surface}; }}\n\
         .diagnostic {{ color: {error}; border-color: {error}; }}\n",
        background = css(palette.background),
        foreground = css(palette.foreground),
        heading = css(palette.heading),
        border = css(palette.border),
        link = css(palette.link),
        muted = css(palette.muted),
        surface = css(palette.surface),
        code = css(palette.code),
        error = css(palette.error),
    );
    sheet
}

pub fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
/// Highlighted `text`, in `theme` or else the light page's InspiredGitHub.
fn code(text: &str, language: &str, theme: Option<&Theme>) -> String {
    static SYNTAX: OnceLock<SyntaxSet> = OnceLock::new();
    static THEMES: OnceLock<ThemeSet> = OnceLock::new();
    let syntax = SYNTAX.get_or_init(SyntaxSet::load_defaults_newlines);
    let theme = theme
        .unwrap_or_else(|| &THEMES.get_or_init(ThemeSet::load_defaults).themes["InspiredGitHub"]);
    let language = match language {
        "javascript" => "js",
        "typescript" => "ts",
        "python" => "py",
        "rust" => "rs",
        x => x,
    };
    let grammar = syntax
        .find_syntax_by_token(language)
        .unwrap_or_else(|| syntax.find_syntax_plain_text());
    syntect::html::highlighted_html_for_string(text, syntax, grammar, theme)
        .unwrap_or_else(|_| format!("<pre>{}</pre>", escape(text)))
}
/// A document ready for layout.
pub struct Page {
    pub html: String,
    /// Whether the page took the editor theme. Authored HTML and a valid SVG
    /// never do, and leave whatever they do not paint transparent.
    pub themed: bool,
}
/// The page for `text`. `palette` themes generated pages only: authored HTML
/// and a valid SVG are shown as written.
pub fn document(text: &str, language: &str, palette: Option<&Palette>) -> Page {
    let themed = palette.map(syntect_theme);
    let theme = themed.as_ref();
    if language == "svg" {
        let error = match roxmltree::Document::parse(text) {
            Ok(doc) if doc.root_element().tag_name().name() == "svg" => None,
            Ok(_) => Some("expected an svg root element".to_owned()),
            Err(error) => Some(error.to_string()),
        };
        // A drawing keeps the white ground it was authored against; only the
        // diagnostic page Runyte generates for a broken one takes the theme.
        let themed = error.is_some() && palette.is_some();
        let (body, ground) = match (error, palette) {
            (Some(error), palette) => (
                format!(
                    "<p>Invalid SVG: {}</p>{}",
                    escape(&error),
                    code(text, "xml", theme)
                ),
                palette.map_or_else(
                    || "background:white".to_owned(),
                    |p| {
                        format!(
                            "background:{};color:{}",
                            css(p.background),
                            css(p.foreground)
                        )
                    },
                ),
            ),
            (None, _) => (text.to_owned(), "background:white".to_owned()),
        };
        return Page {
            html: format!(
                "<!doctype html><html><body style=\"margin:0;{ground}\">{body}</body></html>"
            ),
            themed,
        };
    }
    if language == "html" {
        return Page {
            html: text.to_owned(),
            themed: false,
        };
    }
    let body = match language {
        "markdown" => {
            let parser = Parser::new_ext(
                text,
                Options::ENABLE_TABLES
                    | Options::ENABLE_TASKLISTS
                    | Options::ENABLE_STRIKETHROUGH
                    | Options::ENABLE_GFM,
            );
            let mut events = Vec::new();
            let mut fence: Option<(String, String)> = None;
            for event in parser {
                match event {
                    Event::Start(Tag::CodeBlock(kind)) => {
                        let lang = match kind {
                            pulldown_cmark::CodeBlockKind::Fenced(s) => s.to_string(),
                            _ => String::new(),
                        };
                        fence = Some((lang, String::new()));
                    }
                    Event::End(TagEnd::CodeBlock) => {
                        if let Some((lang, text)) = fence.take() {
                            events.push(Event::Html(code(&text, &lang, theme).into()));
                        }
                    }
                    Event::Text(text) if fence.is_some() => {
                        fence.as_mut().unwrap().1.push_str(&text)
                    }
                    // Raw Markdown HTML is text, never an active document element.
                    Event::Html(text) | Event::InlineHtml(text) => events.push(Event::Text(text)),
                    event => events.push(event),
                }
            }
            let mut result = String::new();
            pulldown_cmark::html::push_html(&mut result, events.into_iter());
            result
        }
        "json" => match serde_json::from_str::<serde_json::Value>(text) {
            Ok(value) => code(
                &serde_json::to_string_pretty(&value).unwrap(),
                "json",
                theme,
            ),
            Err(error) => format!(
                "<p class=diagnostic>Invalid JSON: {}</p>{}",
                escape(&error.to_string()),
                code(text, "json", theme)
            ),
        },
        _ => code(text, language, theme),
    };
    Page {
        html: format!(
            "<!doctype html><html><head><style>{}{}</style></head><body class=markdown-body>{body}</body></html>",
            include_str!("../preview.css"),
            palette.map(themed_stylesheet).unwrap_or_default(),
        ),
        themed: palette.is_some(),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn text_is_escaped() {
        let html = document("<script>alert(1)</script>&", "unknown", None).html;
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;"));
    }
    #[test]
    fn yaml_preserves_structure() {
        let text = "# comment\na: &x [one, two]\nb: *x\n";
        let html = document(text, "yaml", None).html;
        let doc = blitz_html::HtmlDocument::from_html(&html, Default::default());
        let pre = doc.query_selector("pre").unwrap().unwrap();
        assert_eq!(doc.get_node(pre).unwrap().text_content(), text);
    }
    #[test]
    fn invalid_json_falls_back() {
        let html = document("{broken}", "json", None).html;
        assert!(html.contains("Invalid JSON:"));
        let doc = blitz_html::HtmlDocument::from_html(&html, Default::default());
        let pre = doc.query_selector("pre").unwrap().unwrap();
        assert_eq!(doc.get_node(pre).unwrap().text_content(), "{broken}");
    }
    #[test]
    fn valid_json_is_pretty() {
        let html = document("{\"a\":1}", "json", None).html;
        let doc = blitz_html::HtmlDocument::from_html(&html, Default::default());
        let pre = doc.query_selector("pre").unwrap().unwrap();
        assert_eq!(
            doc.get_node(pre).unwrap().text_content(),
            "{\n  \"a\": 1\n}"
        );
        assert!(!html.contains("Invalid JSON"));
    }
    #[test]
    fn malformed_svg_keeps_source_and_diagnostic() {
        let html = document("<svg><path", "svg", None).html;
        assert!(html.contains("Invalid SVG:"));
        let doc = blitz_html::HtmlDocument::from_html(&html, Default::default());
        let pre = doc.query_selector("pre").unwrap().unwrap();
        assert_eq!(doc.get_node(pre).unwrap().text_content(), "<svg><path");
    }
    #[test]
    fn html_is_document() {
        assert_eq!(
            document("<h1>hello</h1>", "html", None).html,
            "<h1>hello</h1>"
        );
    }
    #[test]
    fn markdown_features() {
        let html = document(
            "# Title\n\n- [x] task\n\n|a|b|\n|-|-|\n|1|2|\n\n<script>x</script>",
            "markdown",
            None,
        )
        .html;
        for expected in ["<h1>", "<table>", "checkbox", "&lt;script&gt;"] {
            assert!(html.contains(expected), "{expected}");
        }
    }
    fn palette() -> Palette {
        serde_json::from_value(serde_json::json!({
            "dark": true,
            "background": [0x10, 0x20, 0x30],
            "foreground": [0xee, 0xee, 0xe0],
            "muted": [0x80, 0x80, 0x80],
            "surface": [0x18, 0x28, 0x38],
            "code": [0x20, 0x30, 0x40],
            "border": [0x40, 0x50, 0x60],
            "link": [0x61, 0xaf, 0xef],
            "heading": [0xe5, 0xc0, 0x7b],
            "error": [0xe0, 0x6c, 0x75],
            "syntax": [["keyword", [0xc6, 0x78, 0xdd]], ["unknown", [1, 2, 3]]],
        }))
        .unwrap()
    }
    #[test]
    fn every_scope_selector_parses() {
        for (scope, selectors) in SCOPE_SELECTORS {
            assert!(ScopeSelectors::from_str(selectors).is_ok(), "{scope}");
        }
    }
    #[test]
    fn themed_markdown_uses_palette_and_light_page_is_unchanged() {
        let text = "# Title\n\n[link](x) `inline`\n\n```rust\nfn main() {}\n```\n";
        let light = document(text, "markdown", None).html;
        let themed = document(text, "markdown", Some(&palette()));
        assert!(themed.themed);
        let themed = themed.html;
        for colour in [
            "#102030", "#eeeee0", "#61afef", "#e5c07b", "#405060", "#203040",
        ] {
            assert!(themed.contains(colour), "{colour}");
            assert!(!light.contains(colour), "{colour}");
        }
        // Code blocks sit on the surface and keywords take the theme colour.
        assert!(themed.contains("background-color:#182838"));
        assert!(themed.contains("color:#c678dd"));
        assert!(light.contains("background-color:#ffffff"));
        let doc = blitz_html::HtmlDocument::from_html(&themed, Default::default());
        let pre = doc.query_selector("pre").unwrap().unwrap();
        assert_eq!(doc.get_node(pre).unwrap().text_content(), "fn main() {}\n");
    }
    #[test]
    fn themed_source_and_diagnostics_use_palette() {
        let themed = document("{broken}", "json", Some(&palette())).html;
        assert!(themed.contains(".diagnostic { color: #e06c75"));
        assert!(themed.contains("background-color:#182838"));
        let source = document("fn main() {}", "rust", Some(&palette())).html;
        assert!(source.contains("html { background: #102030; color: #eeeee0; }"));
        assert!(source.contains("color:#c678dd"));
    }
    #[test]
    fn authored_html_and_drawings_keep_their_appearance() {
        let palette = palette();
        assert_eq!(
            document("<h1>hello</h1>", "html", Some(&palette)).html,
            "<h1>hello</h1>"
        );
        let svg = "<svg xmlns=\"http://www.w3.org/2000/svg\"/>";
        assert_eq!(
            document(svg, "svg", Some(&palette)).html,
            document(svg, "svg", None).html
        );
        assert!(!document("<h1>hello</h1>", "html", Some(&palette)).themed);
        assert!(!document(svg, "svg", Some(&palette)).themed);
        assert!(!document("# hi", "markdown", None).themed);
        assert!(document("<svg><path", "svg", Some(&palette)).themed);
        let broken = document("<svg><path", "svg", Some(&palette)).html;
        assert!(broken.contains("background:#102030;color:#eeeee0"));
        assert!(
            document("<svg><path", "svg", None)
                .html
                .contains("background:white")
        );
    }
}
