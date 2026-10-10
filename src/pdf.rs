use anyhow::{anyhow, Context, Result};
use headless_chrome::types::PrintToPdfOptions;
use headless_chrome::{Browser, LaunchOptionsBuilder};
use std::path::{Path, PathBuf};
use tempfile::Builder;

pub struct PdfRenderer;

impl PdfRenderer {
    /// Detects installed Chromium, Google Chrome, Brave, or Microsoft Edge binaries.
    pub fn detect_browser() -> Option<PathBuf> {
        // 1. Explicit environment variables
        for env_var in &["CHROME", "BROWSER_PATH", "CHROME_BIN", "CHROMIUM_PATH"] {
            if let Ok(path_str) = std::env::var(env_var) {
                let p = PathBuf::from(path_str);
                if p.exists() {
                    return Some(p);
                }
            }
        }

        // 2. macOS common application paths
        #[cfg(target_os = "macos")]
        {
            let mac_candidates = [
                "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
                "/Applications/Chromium.app/Contents/MacOS/Chromium",
                "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser",
                "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
                "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary",
            ];
            for path_str in mac_candidates {
                let p = PathBuf::from(path_str);
                if p.exists() {
                    return Some(p);
                }
            }
        }

        // 3. System PATH lookup
        let binaries = [
            "google-chrome",
            "google-chrome-stable",
            "chromium",
            "chromium-browser",
            "brave",
            "brave-browser",
            "microsoft-edge",
            "microsoft-edge-stable",
            "chrome",
        ];
        for bin in binaries {
            if let Ok(p) = which::which(bin) {
                return Some(p);
            }
        }

        // 4. Windows standard program files locations
        #[cfg(target_os = "windows")]
        {
            let win_candidates = [
                r"C:\Program Files\Google\Chrome\Application\chrome.exe",
                r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
                r"C:\Program Files\BraveSoftware\Brave-Browser\Application\brave.exe",
                r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
                r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
            ];
            for path_str in win_candidates {
                let p = PathBuf::from(path_str);
                if p.exists() {
                    return Some(p);
                }
            }
        }

        None
    }

    /// Renders the given HTML string to a Kindle Scribe-formatted PDF.
    pub fn render_html_to_pdf(html: &str, output_path: &Path) -> Result<()> {
        let browser_path = Self::detect_browser();

        // Write sanitized HTML to a temporary file
        let temp_file = Builder::new()
            .prefix("id2scribe-")
            .suffix(".html")
            .tempfile()
            .context("Failed to create temporary HTML file for printing")?;

        std::fs::write(temp_file.path(), html)
            .context("Failed to write HTML content to temporary file")?;

        let canonical_temp_path = temp_file
            .path()
            .canonicalize()
            .context("Failed to canonicalize temporary HTML path")?;
        let file_url = format!("file://{}", canonical_temp_path.display());

        // Configure headless Chrome launch options
        let mut builder = LaunchOptionsBuilder::default();
        builder.headless(true).sandbox(false);

        if let Some(ref path) = browser_path {
            builder.path(Some(path.clone()));
        }

        let launch_options = builder
            .build()
            .map_err(|e| anyhow!("Failed to configure browser launch options: {}", e))?;

        let browser = Browser::new(launch_options).context(
            "Failed to launch headless Chrome/Chromium. Ensure Google Chrome, Chromium, or Brave is installed.",
        )?;

        let tab = browser
            .new_tab()
            .context("Failed to open a new tab in headless browser")?;

        tab.navigate_to(&file_url)
            .with_context(|| format!("Failed to navigate browser to {}", file_url))?;

        tab.wait_until_navigated()
            .context("Browser timed out waiting for page navigation")?;

        // Scribe page specifications: 158 mm x 210 mm
        let scribe_width_inches = 158.0 / 25.4;
        let scribe_height_inches = 210.0 / 25.4;

        let pdf_options = PrintToPdfOptions {
            landscape: Some(false),
            display_header_footer: Some(false),
            print_background: Some(true),
            scale: Some(1.0),
            paper_width: Some(scribe_width_inches),
            paper_height: Some(scribe_height_inches),
            margin_top: Some(0.0),
            margin_bottom: Some(0.0),
            margin_left: Some(0.0),
            margin_right: Some(0.0),
            page_ranges: None,
            ignore_invalid_page_ranges: Some(false),
            header_template: None,
            footer_template: None,
            prefer_css_page_size: Some(true),
            transfer_mode: None,
            generate_document_outline: Some(true),
            generate_tagged_pdf: Some(true),
        };

        let pdf_bytes = tab
            .print_to_pdf(Some(pdf_options))
            .context("Failed to generate PDF from headless tab")?;

        // Ensure parent output directory exists
        if let Some(parent) = output_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).with_context(|| {
                    format!("Failed to create destination directory: {:?}", parent)
                })?;
            }
        }

        std::fs::write(output_path, pdf_bytes)
            .with_context(|| format!("Failed to write PDF to {:?}", output_path))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_browser() {
        if let Some(path) = PdfRenderer::detect_browser() {
            assert!(path.exists(), "Detected browser path must exist: {:?}", path);
        }
    }

    #[test]
    fn test_detect_browser_env_override() {
        let temp_dir = tempfile::tempdir().unwrap();
        let fake_chrome = temp_dir.path().join("fake-chrome-bin");
        std::fs::write(&fake_chrome, "").unwrap();

        // Safety: tests run serially or with unique env var per test if possible,
        // but testing that an existing file is honored by detect_browser
        unsafe {
            std::env::set_var("CHROME", &fake_chrome);
        }
        let detected = PdfRenderer::detect_browser();
        unsafe {
            std::env::remove_var("CHROME");
        }

        assert_eq!(detected, Some(fake_chrome));
    }
}
