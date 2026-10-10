use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_cli_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_id2scribe"))
        .arg("--help")
        .output()
        .expect("Failed to execute id2scribe --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Kindle Scribe"));
    assert!(stdout.contains("--margin-right"));
    assert!(stdout.contains("--font-size"));
}

#[test]
fn test_cli_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_id2scribe"))
        .arg("--version")
        .output()
        .expect("Failed to execute id2scribe --version");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("id2scribe"));
}

#[test]
fn test_cli_missing_arg_fails() {
    let output = Command::new(env!("CARGO_BIN_EXE_id2scribe"))
        .output()
        .expect("Failed to execute id2scribe without args");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Usage:"));
}

#[test]
fn test_cli_convert_local_html_end_to_end() {
    let temp_dir = tempdir().expect("Failed to create tempdir");
    let input_html_path = temp_dir.path().join("sample-draft.html");
    let output_pdf_path = temp_dir.path().join("output-scribe.pdf");

    let sample_html = r##"<!DOCTYPE html>
<html>
<head>
    <title>Sample Internet-Draft</title>
</head>
<body>
    <div id="navbar" class="noprint"><a href="#">Home</a></div>
    <h1>Internet-Draft Sample</h1>
    <p>This is a paragraph verifying Kindle Scribe conversion.</p>
    <pre class="artwork">+----------------------------------------------------------------------+
| 123456789012345678901234567890123456789012345678901234567890123456789012 |
+----------------------------------------------------------------------+</pre>
</body>
</html>"##;

    std::fs::write(&input_html_path, sample_html).expect("Failed to write sample HTML");

    let status = Command::new(env!("CARGO_BIN_EXE_id2scribe"))
        .arg(input_html_path.to_str().unwrap())
        .arg("-o")
        .arg(output_pdf_path.to_str().unwrap())
        .arg("--margin-right")
        .arg("38")
        .arg("--font-size")
        .arg("9.5")
        .status()
        .expect("Failed to execute id2scribe");

    assert!(status.success(), "id2scribe execution must succeed on local HTML");
    assert!(output_pdf_path.exists(), "Output PDF must exist");

    let pdf_bytes = std::fs::read(&output_pdf_path).expect("Failed to read output PDF");
    assert!(!pdf_bytes.is_empty(), "PDF must not be empty");
    assert!(
        pdf_bytes.starts_with(b"%PDF-"),
        "PDF must start with %PDF- header magic"
    );
}
