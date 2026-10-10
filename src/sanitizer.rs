pub struct SanitizerOptions {
    pub margin_right_mm: f32,
    pub font_size_pt: f32,
    pub canonical_url: Option<String>,
}

impl Default for SanitizerOptions {
    fn default() -> Self {
        Self {
            margin_right_mm: 38.0,
            font_size_pt: 9.5,
            canonical_url: None,
        }
    }
}

pub struct Sanitizer;

impl Sanitizer {
    /// Sanitizes HTML and injects Kindle Scribe custom @page and e-ink CSS styles.
    pub fn sanitize_and_inject(html_content: &str, options: &SanitizerOptions) -> String {
        let mono_font_size_pt = (options.font_size_pt * (6.8 / 9.5) * 100.0).round() / 100.0;
        let generated_css = Self::build_css(
            options.margin_right_mm,
            options.font_size_pt,
            mono_font_size_pt,
        );

        // Parse HTML with scraper to inspect and clean DOM elements
        let mut document = html_content.to_string();

        // 1. Remove script tags to avoid background script execution and network requests
        document = Self::strip_tags(&document, "script");

        // 2. Protect preformatted body text in older RFC drafts that wrap the main pre in <div class="rfcmarkup">
        // Since `.rfcmarkup` has `display: none !important;`, we rename any rfcmarkup div that contains pre to rfc-body-wrapper.
        document = Self::protect_rfcmarkup_pre(&document);

        // 3. Inject base tag if canonical URL is provided and not already present
        if let Some(ref url) = options.canonical_url {
            if !document.contains("<base ") && !document.contains("<base>") {
                let base_tag = format!("<base href=\"{}\">\n", url);
                if let Some(pos) = document.find("<head>") {
                    document.insert_str(pos + 6, &format!("\n{}", base_tag));
                } else if let Some(pos) = document.find("<head ") {
                    if let Some(end) = document[pos..].find('>') {
                        document.insert_str(pos + end + 1, &format!("\n{}", base_tag));
                    }
                }
            }
        }

        // 4. Inject styles into <head> or prepend to document
        let style_tag = format!("\n<style id=\"id2scribe-styles\">\n{}\n</style>\n", generated_css);

        if let Some(pos) = document.find("</head>") {
            document.insert_str(pos, &style_tag);
        } else if let Some(pos) = document.find("<body") {
            document.insert_str(pos, &style_tag);
        } else {
            document.push_str(&style_tag);
        }

        document
    }

    /// Generates the exact CSS required for the Kindle Scribe layout.
    pub fn build_css(margin_right_mm: f32, font_size_pt: f32, mono_font_size_pt: f32) -> String {
        format!(
            r#"/* === id2scribe Kindle Scribe Styles === */
@page {{
    size: 158mm 210mm;
    margin-top: 12mm;
    margin-bottom: 12mm;
    margin-left: 12mm;
    margin-right: {margin_right}mm; /* Dynamic, configured via CLI */
    @bottom-right {{
        content: counter(page);
        font-family: monospace;
        font-size: 8pt;
    }}
}}

body {{
    font-family: "Charis SIL", "Georgia", "DejaVu Serif", serif;
    font-size: {font_size}pt;
    line-height: 1.35;
    color: #000000;
    background-color: #ffffff;
    word-break: normal;
    margin: 0 !important;
    padding: 0 !important;
    max-width: 100% !important;
    width: 100% !important;
}}

/* Ensure content containers do not constrain width beyond @page */
#content, .content, main, article {{
    max-width: 100% !important;
    width: 100% !important;
    margin-left: 0 !important;
    margin-right: 0 !important;
    padding-left: 0 !important;
    padding-right: 0 !important;
}}

/* Constrain preformatted blocks and protocol packet diagrams */
pre, .artwork, code {{
    font-family: "DejaVu Sans Mono", "Courier Prime", "Courier New", monospace !important;
    font-size: {mono_font_size}pt !important;
    line-height: 1.15 !important;
    white-space: pre !important;
    overflow: visible !important;
    word-wrap: normal !important;
}}

/* Fit diagram SVGs to content width */
svg, .artwork svg, figure svg {{
    max-width: 100% !important;
    height: auto !important;
}}

/* Strip Datatracker web chrome */
.noprint, #navbar, .tracker, .rfcmarkup, .metadata, nav.navbar, header.navbar {{
    display: none !important;
}}

/* Page break sanity */
h1, h2, h3, h4 {{
    page-break-after: avoid;
    break-after: avoid;
}}

table, figure, .artwork {{
    page-break-inside: avoid;
    break-inside: avoid;
}}
"#,
            margin_right = margin_right_mm,
            font_size = font_size_pt,
            mono_font_size = mono_font_size_pt,
        )
    }

    /// Strips specified HTML tags (e.g. <script>...</script>)
    fn strip_tags(html: &str, tag_name: &str) -> String {
        let open_pattern = format!("<{}", tag_name);
        let close_pattern = format!("</{}>", tag_name);
        let mut result = String::with_capacity(html.len());
        let mut remainder = html;

        while let Some(start_idx) = remainder.to_lowercase().find(&open_pattern) {
            // Check character immediately following tag name is whitespace or '>' or '/'
            let after_tag = start_idx + open_pattern.len();
            if after_tag < remainder.len() {
                let next_char = remainder.as_bytes()[after_tag];
                if next_char != b' ' && next_char != b'\t' && next_char != b'\n' && next_char != b'\r' && next_char != b'>' && next_char != b'/' {
                    result.push_str(&remainder[..after_tag]);
                    remainder = &remainder[after_tag..];
                    continue;
                }
            }

            result.push_str(&remainder[..start_idx]);
            let after_open = &remainder[start_idx..];
            if let Some(close_idx) = after_open.to_lowercase().find(&close_pattern) {
                remainder = &after_open[close_idx + close_pattern.len()..];
            } else {
                // If unclosed tag, skip remainder or self-closing
                if let Some(tag_end) = after_open.find('>') {
                    remainder = &after_open[tag_end + 1..];
                } else {
                    remainder = "";
                }
            }
        }
        result.push_str(remainder);
        result
    }

    /// Protects <div class="rfcmarkup"> from being hidden when it wraps the document <pre>.
    fn protect_rfcmarkup_pre(html: &str) -> String {
        // Simple and robust replacement of class="rfcmarkup" when enclosing content
        let mut out = html.to_string();
        if out.contains("class=\"rfcmarkup\"") && out.contains("<pre") {
            out = out.replace("class=\"rfcmarkup\"", "class=\"rfc-body-wrapper\"");
            out = out.replace("class='rfcmarkup'", "class='rfc-body-wrapper'");
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_css_generation() {
        let css = Sanitizer::build_css(38.0, 9.5, 6.8);
        assert!(css.contains("size: 158mm 210mm;"));
        assert!(css.contains("margin-right: 38mm;"));
        assert!(css.contains("font-size: 9.5pt;"));
        assert!(css.contains("font-size: 6.8pt !important;"));
        assert!(css.contains(".noprint, #navbar, .tracker, .rfcmarkup, .metadata"));
    }

    #[test]
    fn test_script_stripping() {
        let html = "<html><head><script src='test.js'>alert(1);</script></head><body><h1>Hello</h1></body></html>";
        let cleaned = Sanitizer::strip_tags(html, "script");
        assert!(!cleaned.contains("alert(1)"));
        assert!(cleaned.contains("<h1>Hello</h1>"));
    }

    #[test]
    fn test_style_injection() {
        let html = "<html><head><title>Test</title></head><body><p>Content</p></body></html>";
        let opts = SanitizerOptions {
            margin_right_mm: 40.0,
            font_size_pt: 10.0,
            canonical_url: Some("https://example.com/draft.html".to_string()),
        };
        let output = Sanitizer::sanitize_and_inject(html, &opts);
        assert!(output.contains("<style id=\"id2scribe-styles\">"));
        assert!(output.contains("margin-right: 40mm;"));
        assert!(output.contains("<base href=\"https://example.com/draft.html\">"));
    }
}
