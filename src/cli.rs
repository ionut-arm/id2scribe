use clap::Parser;
use std::path::PathBuf;

/// Convert IETF Internet-Drafts into Kindle Scribe PDFs optimized for e-ink reading and stylus annotation.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "id2scribe",
    author,
    version,
    about = "Converts IETF Internet-Drafts to Kindle Scribe-formatted PDFs with stylus margins."
)]
pub struct Cli {
    /// Draft name (e.g. 'draft-ietf-rats-architecture'), versioned ID ('draft-ietf-rats-architecture-22'), or canonical URL
    #[arg(value_name = "DRAFT")]
    pub draft: String,

    /// Destination PDF file path (defaults to <draft-name>.pdf)
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Custom annotation right margin width in mm (default: 38)
    #[arg(long = "margin-right", default_value_t = 38.0, value_name = "MM")]
    pub margin_right: f32,

    /// Base body text font size in pt (default: 9.5)
    #[arg(long = "font-size", default_value_t = 9.5, value_name = "PT")]
    pub font_size: f32,
}
