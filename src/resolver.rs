use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ResolvedDraft {
    pub draft_name: String,
    pub canonical_url: String,
    pub html: String,
}

#[derive(Deserialize, Debug)]
struct DatatrackerDocResponse {
    name: Option<String>,
    rev: Option<String>,
}

pub struct Resolver {
    client: Client,
}

impl Resolver {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent("id2scribe/0.1.0 (IETF to Kindle Scribe PDF Converter)")
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .context("Failed to build HTTP client")?;
        Ok(Self { client })
    }

    /// Resolves the user-provided draft identifier or URL and retrieves the HTML content.
    pub async fn resolve_and_fetch(&self, draft_input: &str) -> Result<ResolvedDraft> {
        // 1. Check if the input is a local file
        let path = Path::new(draft_input);
        if path.exists() && path.is_file() {
            let html = tokio::fs::read_to_string(path)
                .await
                .with_context(|| format!("Failed to read local file: {}", draft_input))?;
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("draft")
                .to_string();
            let canonical_url = format!("file://{}", path.canonicalize()?.display());
            return Ok(ResolvedDraft {
                draft_name: stem,
                canonical_url,
                html,
            });
        }

        // 2. Extract base identifier from URL or raw name
        let extracted_id = Self::extract_draft_id(draft_input);

        // 3. Determine if the identifier already includes a revision number (e.g., ends with -XX)
        let full_id = if Self::is_versioned(&extracted_id) {
            extracted_id.clone()
        } else {
            // Unversioned: query Datatracker API to find latest revision
            self.resolve_latest_revision(&extracted_id).await?
        };

        // 4. Retrieve canonical HTML
        let (html, canonical_url) = self.fetch_canonical_html(&full_id).await?;

        Ok(ResolvedDraft {
            draft_name: full_id,
            canonical_url,
            html,
        })
    }

    /// Extracts clean draft identifier from input (URL, filename, or draft slug).
    pub fn extract_draft_id(input: &str) -> String {
        let trimmed = input.trim();
        let stripped_url = if let Some(idx) = trimmed.find("://") {
            &trimmed[idx + 3..]
        } else {
            trimmed
        };

        // Strip query string and fragment
        let clean = stripped_url
            .split('?')
            .next()
            .unwrap_or(stripped_url)
            .split('#')
            .next()
            .unwrap_or(stripped_url)
            .trim_end_matches('/');

        // Get last path segment
        let segment = clean.split('/').last().unwrap_or(clean);

        // Strip file extensions like .html, .txt, .pdf, .xml
        let without_ext = if let Some(stripped) = segment.strip_suffix(".html") {
            stripped
        } else if let Some(stripped) = segment.strip_suffix(".txt") {
            stripped
        } else if let Some(stripped) = segment.strip_suffix(".xml") {
            stripped
        } else if let Some(stripped) = segment.strip_suffix(".pdf") {
            stripped
        } else {
            segment
        };

        without_ext.to_string()
    }

    /// Checks if the identifier already has a two-digit revision suffix like `-22` or `-00`.
    pub fn is_versioned(id: &str) -> bool {
        if let Some(pos) = id.rfind('-') {
            let suffix = &id[pos + 1..];
            suffix.len() == 2 && suffix.chars().all(|c| c.is_ascii_digit())
        } else {
            false
        }
    }

    /// Queries Datatracker API to resolve unversioned draft names to the latest revision.
    pub async fn resolve_latest_revision(&self, draft_name: &str) -> Result<String> {
        let api_url = format!(
            "https://datatracker.ietf.org/api/v1/doc/document/{}/?format=json",
            draft_name
        );

        match self.client.get(&api_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(doc) = resp.json::<DatatrackerDocResponse>().await {
                    if let (Some(name), Some(rev)) = (doc.name, doc.rev) {
                        return Ok(format!("{}-{}", name, rev));
                    }
                }
            }
            _ => {}
        }

        // Fallback: Scrape https://datatracker.ietf.org/doc/html/<name>
        let dt_url = format!("https://datatracker.ietf.org/doc/html/{}", draft_name);
        if let Ok(resp) = self.client.get(&dt_url).send().await {
            // Check final redirected URL first
            let final_url = resp.url().to_string();
            let extracted = Self::extract_draft_id(&final_url);
            if Self::is_versioned(&extracted) {
                return Ok(extracted);
            }

            // Fallback: scrape HTML for draft name
            if let Ok(text) = resp.text().await {
                if let Some(rev) = Self::scrape_draft_version(&text, draft_name) {
                    return Ok(rev);
                }
            }
        }

        // If unable to query datatracker, return unversioned identifier as fallback
        Ok(draft_name.to_string())
    }

    fn scrape_draft_version(html: &str, base_name: &str) -> Option<String> {
        // Check for <meta name="ietf.draft" content="draft-...-XX">
        let meta_marker = "name=\"ietf.draft\" content=\"";
        if let Some(pos) = html.find(meta_marker) {
            let start = pos + meta_marker.len();
            if let Some(end) = html[start..].find('"') {
                let candidate = &html[start..start + end];
                if Self::is_versioned(candidate) {
                    return Some(candidate.to_string());
                }
            }
        }

        // Pattern search for base_name followed by -XX
        let pattern = format!("{}-", base_name);
        for (idx, _) in html.match_indices(&pattern) {
            let candidate_start = idx;
            let suffix_start = idx + pattern.len();
            if suffix_start + 2 <= html.len() {
                let digits = &html[suffix_start..suffix_start + 2];
                if digits.chars().all(|c| c.is_ascii_digit()) {
                    let candidate = &html[candidate_start..suffix_start + 2];
                    return Some(candidate.to_string());
                }
            }
        }

        None
    }

    /// Retrieves canonical HTML from the archive or datatracker.
    async fn fetch_canonical_html(&self, draft_id: &str) -> Result<(String, String)> {
        // 1. Try canonical archive URL: https://www.ietf.org/archive/id/<draft-id>.html
        let archive_url = format!("https://www.ietf.org/archive/id/{}.html", draft_id);
        if let Ok(resp) = self.client.get(&archive_url).send().await {
            if resp.status().is_success() {
                if let Ok(text) = resp.text().await {
                    if !text.trim().is_empty() {
                        return Ok((text, archive_url));
                    }
                }
            }
        }

        // 2. Fallback to datatracker HTML: https://datatracker.ietf.org/doc/html/<draft-id>
        let dt_url = format!("https://datatracker.ietf.org/doc/html/{}", draft_id);
        let resp = self
            .client
            .get(&dt_url)
            .send()
            .await
            .with_context(|| format!("Failed to request draft from {}", dt_url))?;

        if !resp.status().is_success() {
            return Err(anyhow!(
                "Failed to fetch draft '{}': HTTP status {}",
                draft_id,
                resp.status()
            ));
        }

        let text = resp
            .text()
            .await
            .with_context(|| format!("Failed to read response body for {}", dt_url))?;

        Ok((text, dt_url))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_draft_id() {
        assert_eq!(
            Resolver::extract_draft_id("draft-ietf-rats-architecture"),
            "draft-ietf-rats-architecture"
        );
        assert_eq!(
            Resolver::extract_draft_id("draft-ietf-rats-architecture-22"),
            "draft-ietf-rats-architecture-22"
        );
        assert_eq!(
            Resolver::extract_draft_id("https://datatracker.ietf.org/doc/draft-ietf-rats-architecture/"),
            "draft-ietf-rats-architecture"
        );
        assert_eq!(
            Resolver::extract_draft_id("https://datatracker.ietf.org/doc/html/draft-ietf-rats-architecture-22"),
            "draft-ietf-rats-architecture-22"
        );
        assert_eq!(
            Resolver::extract_draft_id("https://www.ietf.org/archive/id/draft-ietf-rats-architecture-22.html"),
            "draft-ietf-rats-architecture-22"
        );
        assert_eq!(
            Resolver::extract_draft_id("https://www.ietf.org/archive/id/draft-ietf-rats-architecture-22.txt"),
            "draft-ietf-rats-architecture-22"
        );
    }

    #[test]
    fn test_is_versioned() {
        assert!(Resolver::is_versioned("draft-ietf-rats-architecture-22"));
        assert!(Resolver::is_versioned("draft-ietf-rats-architecture-00"));
        assert!(!Resolver::is_versioned("draft-ietf-rats-architecture"));
        assert!(!Resolver::is_versioned("draft-ietf-rats-architecture-"));
        assert!(!Resolver::is_versioned("draft-ietf-rats-architecture-222"));
    }

    #[test]
    fn test_extract_draft_id_with_query_and_fragment() {
        assert_eq!(
            Resolver::extract_draft_id("https://datatracker.ietf.org/doc/html/draft-ietf-rats-architecture-22?include_text=1#section-3"),
            "draft-ietf-rats-architecture-22"
        );
        assert_eq!(
            Resolver::extract_draft_id("https://datatracker.ietf.org/doc/draft-ietf-rats-architecture-22/?raw=1"),
            "draft-ietf-rats-architecture-22"
        );
    }

    #[test]
    fn test_scrape_draft_version() {
        let html_meta = r#"<html><head><meta name="ietf.draft" content="draft-ietf-rats-architecture-22"></head></html>"#;
        assert_eq!(
            Resolver::scrape_draft_version(html_meta, "draft-ietf-rats-architecture"),
            Some("draft-ietf-rats-architecture-22".to_string())
        );

        let html_text = r#"<title>draft-ietf-quic-transport-34</title>"#;
        assert_eq!(
            Resolver::scrape_draft_version(html_text, "draft-ietf-quic-transport"),
            Some("draft-ietf-quic-transport-34".to_string())
        );

        let html_none = r#"<html><body>No version here</body></html>"#;
        assert_eq!(
            Resolver::scrape_draft_version(html_none, "draft-ietf-rats-architecture"),
            None
        );
    }

    #[tokio::test]
    async fn test_resolve_local_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("draft-local-test-01.html");
        tokio::fs::write(&file_path, "<h1>Local Draft</h1>").await.unwrap();

        let resolver = Resolver::new().unwrap();
        let resolved = resolver
            .resolve_and_fetch(file_path.to_str().unwrap())
            .await
            .unwrap();

        assert_eq!(resolved.draft_name, "draft-local-test-01");
        assert!(resolved.html.contains("<h1>Local Draft</h1>"));
        assert!(resolved.canonical_url.starts_with("file://"));
    }
}
