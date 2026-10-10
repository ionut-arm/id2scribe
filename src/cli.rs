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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_defaults() {
        let args = Cli::try_parse_from(["id2scribe", "draft-ietf-rats-architecture"]).unwrap();
        assert_eq!(args.draft, "draft-ietf-rats-architecture");
        assert_eq!(args.output, None);
        assert_eq!(args.margin_right, 38.0);
        assert_eq!(args.font_size, 9.5);
    }

    #[test]
    fn test_cli_custom_flags() {
        let args = Cli::try_parse_from([
            "id2scribe",
            "draft-ietf-quic-transport-34",
            "-o",
            "custom.pdf",
            "--margin-right",
            "45.5",
            "--font-size",
            "10.0",
        ])
        .unwrap();
        assert_eq!(args.draft, "draft-ietf-quic-transport-34");
        assert_eq!(args.output, Some(PathBuf::from("custom.pdf")));
        assert_eq!(args.margin_right, 45.5);
        assert_eq!(args.font_size, 10.0);
    }

    #[test]
    fn test_cli_missing_draft_error() {
        let err = Cli::try_parse_from(["id2scribe"]);
        assert!(err.is_err());
    }

    #[test]
    fn test_cli_invalid_float_error() {
        let err = Cli::try_parse_from(["id2scribe", "draft-foo", "--margin-right", "not-a-number"]);
        assert!(err.is_err());
    }
}
