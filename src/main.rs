mod cli;
mod pdf;
mod resolver;
mod sanitizer;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use pdf::PdfRenderer;
use resolver::Resolver;
use sanitizer::{Sanitizer, SanitizerOptions};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    println!(">>> id2scribe: IETF Draft to Kindle Scribe PDF Converter");
    println!("Resolving draft: {}", args.draft);

    let resolver = Resolver::new()?;
    let resolved = resolver.resolve_and_fetch(&args.draft).await?;
    println!("Resolved canonical ID: {}", resolved.draft_name);
    println!("Source URL: {}", resolved.canonical_url);

    let output_path = match args.output {
        Some(path) => path,
        None => PathBuf::from(format!("{}.pdf", resolved.draft_name)),
    };

    println!(
        "Sanitizing DOM and injecting Kindle Scribe styles (margin-right: {}mm, font-size: {}pt)...",
        args.margin_right, args.font_size
    );

    let sanitizer_opts = SanitizerOptions {
        margin_right_mm: args.margin_right,
        font_size_pt: args.font_size,
        canonical_url: Some(resolved.canonical_url.clone()),
    };

    let styled_html = Sanitizer::sanitize_and_inject(&resolved.html, &sanitizer_opts);

    println!("Rendering fixed-layout PDF via headless Chromium...");
    PdfRenderer::render_html_to_pdf(&styled_html, &output_path)?;

    let file_size = std::fs::metadata(&output_path)?.len();
    let file_size_kb = file_size as f64 / 1024.0;

    println!(
        "Successfully created Kindle Scribe PDF: {} ({:.1} KB)",
        output_path.display(),
        file_size_kb
    );

    Ok(())
}
