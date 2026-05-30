use std::fs;

pub const DEFAULT_CSS: &str = r#"
:root {
    --color-text: #1a1a1a;
    --color-heading: #111;
    --color-link: #0366d6;
    --color-code-bg: #f6f8fa;
    --color-border: #e1e4e8;
    --color-blockquote-border: #dfe2e5;
    --color-table-border: #dfe2e5;
    --font-body: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif;
    --font-heading: var(--font-body);
    --font-mono: SFMono-Regular, Consolas, "Liberation Mono", Menlo, monospace;
    --max-width: 860px;
    --font-size: 16px;
    --line-height: 1.6;
}

@media print {
    :root {
        --max-width: 100%;
    }
    body { padding: 0; }
}

body {
    font-family: var(--font-body);
    font-size: var(--font-size);
    line-height: var(--line-height);
    color: var(--color-text);
    max-width: var(--max-width);
    margin: 0 auto;
    padding: 2rem 1rem;
}

h1, h2, h3, h4, h5, h6 {
    font-family: var(--font-heading);
    color: var(--color-heading);
    margin-top: 1.5em;
    margin-bottom: 0.5em;
    line-height: 1.25;
}

h1 { font-size: 2em; border-bottom: 1px solid var(--color-border); padding-bottom: 0.3em; }
h2 { font-size: 1.5em; border-bottom: 1px solid var(--color-border); padding-bottom: 0.3em; }
h3 { font-size: 1.25em; }
h4 { font-size: 1em; }

a { color: var(--color-link); text-decoration: none; }
a:hover { text-decoration: underline; }

code {
    font-family: var(--font-mono);
    font-size: 0.85em;
    background: var(--color-code-bg);
    padding: 0.2em 0.4em;
    border-radius: 3px;
}

pre {
    font-family: var(--font-mono);
    font-size: 0.85em;
    background: var(--color-code-bg);
    padding: 1em;
    border-radius: 6px;
    overflow-x: auto;
    border: 1px solid var(--color-border);
}

pre code {
    background: none;
    padding: 0;
    border-radius: 0;
    font-size: inherit;
}

blockquote {
    margin: 0;
    padding: 0.5em 1em;
    border-left: 4px solid var(--color-blockquote-border);
    color: #555;
}

table {
    border-collapse: collapse;
    width: 100%;
    margin: 1em 0;
}

th, td {
    padding: 0.5em 1em;
    border: 1px solid var(--color-table-border);
    text-align: left;
}

th { background: var(--color-code-bg); font-weight: 600; }

img { max-width: 100%; height: auto; }

hr { border: none; border-top: 1px solid var(--color-border); margin: 2em 0; }

ul, ol { padding-left: 2em; }
li { margin: 0.25em 0; }

figure { margin: 1.5em 0; text-align: center; }
figcaption { font-size: 0.9em; color: #555; margin-top: 0.5em; }

.toc {
    background: var(--color-code-bg);
    border: 1px solid var(--color-border);
    padding: 1em 1.5em;
    margin-bottom: 2em;
    border-radius: 6px;
}
.toc h2 { margin-top: 0; font-size: 1.2em; }
.toc ul { list-style: none; padding-left: 0; }
.toc ul ul { padding-left: 1.5em; }
.toc a { color: var(--color-text); }

.math-display { display: block; text-align: center; margin: 1em 0; padding: 0.5em; overflow-x: auto; }
.math { font-family: serif; font-style: italic; }
.eq-number { float: right; }
.ref { color: var(--color-link); }

.footnote-ref { font-size: 0.8em; vertical-align: super; }
.footnotes { font-size: 0.9em; border-top: 1px solid var(--color-border); margin-top: 2em; padding-top: 1em; }
.footnotes li { margin-bottom: 0.3em; }

.figure { margin: 1em 0; text-align: center; }
.figure img { display: block; margin: 0 auto; }
.caption { font-size: 0.9em; color: #555; margin-top: 0.3em; }

.anchor-link {
    color: var(--color-link);
    text-decoration: none;
    margin-left: 0.2em;
    opacity: 0;
    transition: opacity 0.2s;
}
:hover > .anchor-link { opacity: 1; }
"#;

pub const GITHUB_CSS: &str = r#"
:root {
    --color-text: #24292f;
    --color-heading: #1f2328;
    --color-link: #0969da;
    --color-code-bg: #eff1f3;
    --color-border: #d0d7de;
    --color-blockquote-border: #d0d7de;
    --color-table-border: #d0d7de;
    --font-body: -apple-system, BlinkMacSystemFont, "Segoe UI", "Noto Sans", Helvetica, Arial, sans-serif;
    --font-heading: -apple-system, BlinkMacSystemFont, "Segoe UI", "Noto Sans", Helvetica, Arial, sans-serif;
    --font-mono: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, "Liberation Mono", monospace;
    --max-width: 980px;
    --font-size: 16px;
    --line-height: 1.5;
}

@media print {
    :root { --max-width: 100%; }
    body { padding: 0; }
}

body {
    font-family: var(--font-body);
    font-size: var(--font-size);
    line-height: var(--line-height);
    color: var(--color-text);
    max-width: var(--max-width);
    margin: 0 auto;
    padding: 2rem 1rem;
}

h1, h2, h3, h4, h5, h6 {
    font-family: var(--font-heading);
    color: var(--color-heading);
    margin-top: 24px;
    margin-bottom: 16px;
    font-weight: 600;
    line-height: 1.25;
}

h1 {
    font-size: 2em;
    padding-bottom: 0.3em;
    border-bottom: 1px solid var(--color-border);
}
h2 {
    font-size: 1.5em;
    padding-bottom: 0.3em;
    border-bottom: 1px solid var(--color-border);
}
h3 { font-size: 1.25em; }
h4 { font-size: 1em; }

a { color: var(--color-link); text-decoration: none; }
a:hover { text-decoration: underline; }

code {
    font-family: var(--font-mono);
    font-size: 85%;
    padding: 0.2em 0.4em;
    background: var(--color-code-bg);
    border-radius: 6px;
}

pre {
    font-family: var(--font-mono);
    font-size: 85%;
    line-height: 1.45;
    background: var(--color-code-bg);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    padding: 16px;
    overflow: auto;
}

pre code {
    background: none;
    padding: 0;
    border-radius: 0;
    font-size: 100%;
}

blockquote {
    margin: 0;
    padding: 0 1em;
    color: var(--color-text);
    border-left: 0.25em solid var(--color-blockquote-border);
}

table {
    border-collapse: collapse;
    width: 100%;
    overflow-x: auto;
    display: block;
    margin: 1em 0;
}

th, td {
    padding: 6px 13px;
    border: 1px solid var(--color-table-border);
}

th {
    font-weight: 600;
    background: var(--color-code-bg);
}

img { max-width: 100%; box-sizing: content-box; }

hr { border: 0; border-top: 1px solid var(--color-border); margin: 2em 0; }

ul, ol { padding-left: 2em; }
li { margin: 0.25em 0; }

.toc {
    background: var(--color-code-bg);
    border: 1px solid var(--color-border);
    padding: 16px 24px;
    margin-bottom: 2em;
    border-radius: 6px;
}
.toc h2 { margin-top: 0; font-size: 1.25em; border: none; padding: 0; }
.toc ul { list-style: none; padding-left: 0; }
.toc ul ul { padding-left: 1.5em; }
.toc a { color: var(--color-text); }

.math-display { display: block; text-align: center; margin: 1em 0; padding: 0.5em; overflow-x: auto; }
.math { font-family: serif; font-style: italic; }
.eq-number { float: right; }
.ref { color: var(--color-link); }

.footnote-ref { font-size: 0.8em; vertical-align: super; }
.footnotes { font-size: 0.9em; border-top: 1px solid var(--color-border); margin-top: 2em; padding-top: 1em; }
.footnotes li { margin-bottom: 0.3em; }

.figure { margin: 1em 0; text-align: center; }
.figure img { display: block; margin: 0 auto; }
.caption { font-size: 0.9em; color: #57606a; margin-top: 0.3em; }

.anchor-link {
    color: var(--color-link);
    text-decoration: none;
    margin-left: 0.2em;
    opacity: 0;
    transition: opacity 0.2s;
}
:hover > .anchor-link { opacity: 1; }
"#;

pub const LATEX_CSS: &str = r#"
:root {
    --color-text: #000;
    --color-heading: #000;
    --color-link: #1a0dab;
    --color-code-bg: #f5f5f5;
    --color-border: #000;
    --color-blockquote-border: #000;
    --color-table-border: #000;
    --font-body: "Latin Modern Roman", "Times New Roman", "Computer Modern", Georgia, serif;
    --font-heading: var(--font-body);
    --font-mono: "Latin Modern Mono", "Courier New", Courier, monospace;
    --max-width: 680px;
    --font-size: 11pt;
    --line-height: 1.45;
}

@media print {
    :root { --max-width: 100%; }
    body { padding: 0; }
}

body {
    font-family: var(--font-body);
    font-size: var(--font-size);
    line-height: var(--line-height);
    color: var(--color-text);
    max-width: var(--max-width);
    margin: 0 auto;
    padding: 2rem 1rem;
    text-align: justify;
    hyphens: auto;
}

h1, h2, h3, h4, h5, h6 {
    font-family: var(--font-heading);
    color: var(--color-heading);
    margin-top: 1.8em;
    margin-bottom: 0.6em;
    line-height: 1.2;
    font-weight: normal;
}

h1 { font-size: 1.6em; text-align: center; }
h2 { font-size: 1.4em; }
h3 { font-size: 1.2em; }
h4 { font-size: 1.1em; }

a { color: var(--color-link); text-decoration: none; }
a:hover { text-decoration: underline; }

code {
    font-family: var(--font-mono);
    font-size: 0.9em;
    background: var(--color-code-bg);
    padding: 0.1em 0.3em;
}

pre {
    font-family: var(--font-mono);
    font-size: 0.9em;
    background: var(--color-code-bg);
    padding: 1em;
    overflow-x: auto;
    border: 1px solid var(--color-border);
}

pre code {
    background: none;
    padding: 0;
    font-size: inherit;
}

blockquote {
    margin: 1em 0;
    padding: 0.5em 1.5em;
    border-left: 3px solid var(--color-blockquote-border);
    font-style: italic;
}

table {
    border-collapse: collapse;
    width: 100%;
    margin: 1em 0;
    caption-side: bottom;
}

th, td {
    padding: 0.4em 0.8em;
    border: 1px solid var(--color-table-border);
    text-align: center;
}

th { font-weight: bold; }

img { max-width: 100%; height: auto; }

hr { border: none; border-top: 1px solid var(--color-border); margin: 2em 0; }

ul, ol { padding-left: 2em; }
li { margin: 0.3em 0; }

.toc {
    margin-bottom: 2em;
    padding: 1em 0;
}
.toc h2 { margin-top: 0; font-size: 1.2em; text-align: center; }
.toc ul { list-style: none; padding-left: 0; }
.toc ul ul { padding-left: 1.5em; }
.toc a { color: var(--color-text); }

.math-display { display: block; text-align: center; margin: 1em 0; padding: 0.5em; overflow-x: auto; }
.math { font-family: serif; font-style: italic; }
.eq-number { float: right; }
.ref { color: var(--color-link); }

.footnote-ref { font-size: 0.75em; vertical-align: super; }
.footnotes { font-size: 0.85em; margin-top: 2em; padding-top: 0.5em; }
.footnotes li { margin-bottom: 0.3em; }

.figure { margin: 1.5em 0; text-align: center; }
.figure img { display: block; margin: 0 auto; }
.caption { font-size: 0.9em; margin-top: 0.3em; }

.anchor-link {
    color: var(--color-link);
    text-decoration: none;
    margin-left: 0.2em;
    opacity: 0;
    transition: opacity 0.2s;
}
:hover > .anchor-link { opacity: 1; }
"#;

pub const MINIMAL_CSS: &str = r#"
body {
    max-width: 720px;
    margin: 0 auto;
    padding: 2em 1em;
    line-height: 1.6;
    font-family: sans-serif;
}

pre { padding: 1em; overflow-x: auto; background: #f5f5f5; }
code { font-family: monospace; font-size: 0.9em; }
pre code { background: none; padding: 0; }
blockquote { margin-left: 0; padding-left: 1em; border-left: 3px solid #ccc; }
table { border-collapse: collapse; width: 100%; }
th, td { border: 1px solid #ddd; padding: 0.5em 0.8em; }
img { max-width: 100%; }
hr { margin: 2em 0; }
"#;

pub const DARK_CSS: &str = r#"
:root {
    --color-text: #c9d1d9;
    --color-heading: #e6edf3;
    --color-link: #58a6ff;
    --color-code-bg: #161b22;
    --color-border: #30363d;
    --color-blockquote-border: #30363d;
    --color-table-border: #30363d;
    --font-body: -apple-system, BlinkMacSystemFont, "Segoe UI", "Noto Sans", Helvetica, Arial, sans-serif;
    --font-heading: var(--font-body);
    --font-mono: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
    --max-width: 860px;
    --font-size: 16px;
    --line-height: 1.6;
}

@media print {
    :root { --max-width: 100%; --color-text: #000; --color-heading: #000; --color-code-bg: #fff; --color-border: #ccc; }
    body { padding: 0; }
}

@media (prefers-color-scheme: light) {
    :root {
        --color-text: #1a1a1a;
        --color-heading: #111;
        --color-link: #0366d6;
        --color-code-bg: #f6f8fa;
        --color-border: #e1e4e8;
    }
}

body {
    font-family: var(--font-body);
    font-size: var(--font-size);
    line-height: var(--line-height);
    color: var(--color-text);
    background: #0d1117;
    max-width: var(--max-width);
    margin: 0 auto;
    padding: 2rem 1rem;
}

h1, h2, h3, h4, h5, h6 {
    font-family: var(--font-heading);
    color: var(--color-heading);
    margin-top: 1.5em;
    margin-bottom: 0.5em;
    line-height: 1.25;
}

h1 { font-size: 2em; border-bottom: 1px solid var(--color-border); padding-bottom: 0.3em; }
h2 { font-size: 1.5em; border-bottom: 1px solid var(--color-border); padding-bottom: 0.3em; }
h3 { font-size: 1.25em; }
h4 { font-size: 1em; }

a { color: var(--color-link); text-decoration: none; }
a:hover { text-decoration: underline; }

code {
    font-family: var(--font-mono);
    font-size: 0.85em;
    background: var(--color-code-bg);
    padding: 0.2em 0.4em;
    border-radius: 3px;
}

pre {
    font-family: var(--font-mono);
    font-size: 0.85em;
    background: var(--color-code-bg);
    padding: 1em;
    border-radius: 6px;
    overflow-x: auto;
    border: 1px solid var(--color-border);
}

pre code {
    background: none;
    padding: 0;
    border-radius: 0;
    font-size: inherit;
}

blockquote {
    margin: 0;
    padding: 0.5em 1em;
    border-left: 4px solid var(--color-blockquote-border);
    color: #8b949e;
}

table {
    border-collapse: collapse;
    width: 100%;
    margin: 1em 0;
}

th, td {
    padding: 0.5em 1em;
    border: 1px solid var(--color-table-border);
    text-align: left;
}

th { background: var(--color-code-bg); font-weight: 600; }

img { max-width: 100%; height: auto; }

hr { border: none; border-top: 1px solid var(--color-border); margin: 2em 0; }

ul, ol { padding-left: 2em; }
li { margin: 0.25em 0; }

.toc {
    background: var(--color-code-bg);
    border: 1px solid var(--color-border);
    padding: 1em 1.5em;
    margin-bottom: 2em;
    border-radius: 6px;
}
.toc h2 { margin-top: 0; font-size: 1.2em; }
.toc ul { list-style: none; padding-left: 0; }
.toc ul ul { padding-left: 1.5em; }
.toc a { color: var(--color-text); }

.math-display { display: block; text-align: center; margin: 1em 0; padding: 0.5em; overflow-x: auto; }
.math { font-family: serif; font-style: italic; }
.eq-number { float: right; }
.ref { color: var(--color-link); }

.footnote-ref { font-size: 0.8em; vertical-align: super; }
.footnotes { font-size: 0.9em; border-top: 1px solid var(--color-border); margin-top: 2em; padding-top: 1em; }
.footnotes li { margin-bottom: 0.3em; }

.figure { margin: 1em 0; text-align: center; }
.figure img { display: block; margin: 0 auto; }
.caption { font-size: 0.9em; color: #8b949e; margin-top: 0.3em; }

.anchor-link {
    color: var(--color-link);
    text-decoration: none;
    margin-left: 0.2em;
    opacity: 0;
    transition: opacity 0.2s;
}
:hover > .anchor-link { opacity: 1; }
"#;

pub struct HtmlTheme;

impl HtmlTheme {
    pub fn default_theme() -> &'static str {
        DEFAULT_CSS
    }

    pub fn github() -> &'static str {
        GITHUB_CSS
    }

    pub fn latex() -> &'static str {
        LATEX_CSS
    }

    pub fn minimal() -> &'static str {
        MINIMAL_CSS
    }

    pub fn dark() -> &'static str {
        DARK_CSS
    }

    pub fn resolve(name: &str) -> Option<&'static str> {
        match name {
            "default" | "" => Some(DEFAULT_CSS),
            "github" => Some(GITHUB_CSS),
            "latex" => Some(LATEX_CSS),
            "minimal" => Some(MINIMAL_CSS),
            "dark" => Some(DARK_CSS),
            _ => None,
        }
    }
}

pub fn slugify(text: &str) -> String {
    let mut slug = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            'a'..='z' | '0'..='9' => slug.push(c),
            'A'..='Z' => slug.push(c.to_ascii_lowercase()),
            ' ' | '-' => slug.push('-'),
            _ => {}
        }
    }
    let trimmed = slug.trim_matches('-');
    if trimmed.is_empty() {
        "heading".to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn load_custom_css(path: &str) -> Result<String, std::io::Error> {
    fs::read_to_string(path)
}

pub fn generate_toc(headings: &[(u32, String)], max_depth: u32) -> String {
    let mut html = String::from("<nav class=\"toc\">\n<h2>Table of Contents</h2>\n<ul>\n");
    let mut current_depth = 0u32;

    for &(level, ref text) in headings {
        if level > max_depth {
            continue;
        }
        while current_depth < level {
            html.push_str("<ul>\n");
            current_depth += 1;
        }
        while current_depth > level {
            html.push_str("</ul>\n");
            current_depth -= 1;
        }
        let anchor = slugify(text);
        html.push_str(&format!(
            "<li><a href=\"#{}\">{}</a></li>\n",
            escape_html(&anchor),
            escape_html(text)
        ));
    }

    while current_depth > 0 {
        html.push_str("</ul>\n");
        current_depth -= 1;
    }

    html.push_str("</ul>\n</nav>\n");
    html
}

fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}
