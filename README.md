# `id2scribe`

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

**IETF Internet-Draft to Kindle Scribe PDF Converter**

`id2scribe` is a standalone CLI utility written in Rust that fetches IETF Internet-Drafts, adapts their DOM structure, typography, and ASCII/SVG artwork for e-ink reading, and generates fixed-layout PDFs dimensioned specifically for the **Amazon Kindle Scribe** with dedicated wide margins for stylus annotation.

---

## Target Device & Layout Specifications

* **Target Device:** Amazon Kindle Scribe (1860 × 2480 px, 300 ppi, physical viewport ~157.5 mm × 210 mm).
* **Page Dimensions:** Fixed page size of exactly **158 mm width × 210 mm height** (`448.08 pt × 594.96 pt`).
* **Margin Allocation:**
  * **Top:** 12 mm
  * **Bottom:** 12 mm (with dynamic page counter at `@bottom-right`)
  * **Left (binding / inner):** 12 mm
  * **Right (stylus markup area):** 38 mm (configurable via `--margin-right`)
* **Printable Content Width:** 108 mm.
* **Diagram & ASCII Artwork Rule:** Monospace blocks (`<pre>`, `.artwork`, protocol packet headers, ABNF) and SVGs are scaled (`font-size: ~6.8pt`, `line-height: 1.15`, `white-space: pre !important`) so that 72-character lines fit comfortably within the 108 mm content width without horizontal line wrapping or clipping.

---

## Features

* **Flexible Ingestion:**
  * **Unversioned Drafts:** Resolves latest revisions automatically via the IETF Datatracker REST API (`https://datatracker.ietf.org/api/v1/doc/document/<name>/?format=json`) with HTML scraping fallback.
  * **Versioned Drafts:** Fetches specific revisions (e.g. `draft-ietf-rats-architecture-22`).
  * **URLs:** Accepts full Datatracker (`https://datatracker.ietf.org/doc/html/...`) or IETF Archive (`https://www.ietf.org/archive/id/...`) URLs.
  * **Local Files:** Accepts paths to locally saved HTML drafts.
* **DOM Sanitation:** Strips Datatracker web chrome (`#navbar`, `.noprint`, `.tracker`, `.metadata`), removes executing scripts, and preserves body text in preformatted drafts.
* **Headless Chromium Automation:** Leverages the Chrome DevTools Protocol (CDP) via `headless_chrome` with cross-platform browser detection (Google Chrome, Chromium, Brave, Microsoft Edge).

---

## Installation & Build

### Prerequisites
* Rust toolchain (Rust 2024 / 1.85+)
* An installed Chromium-based browser (Google Chrome, Chromium, Brave, or Microsoft Edge)

### Building from Source

```bash
git clone https://github.com/ionut-arm/id2scribe.git
cd id2scribe
cargo build --release
```

The compiled binary will be placed at `target/release/id2scribe`.

To install into your `~/.cargo/bin`:
```bash
cargo install --path .
```

---

## Usage

```bash
# Convert an unversioned draft (automatically resolves latest revision)
id2scribe draft-ietf-rats-architecture

# Convert a specific versioned draft
id2scribe draft-ietf-rats-architecture-22

# Convert from a canonical Datatracker URL to a custom output path
id2scribe https://datatracker.ietf.org/doc/html/draft-ietf-quic-transport-34 -o quic.pdf

# Customize margin width (e.g. 45 mm) and font size (e.g. 10.5 pt)
id2scribe draft-ietf-rats-architecture-22 --margin-right 45 --font-size 10.5
```

### CLI Arguments & Options

| Option / Argument | Default | Description |
|---|---|---|
| `<DRAFT>` | *required* | Draft name (e.g. `draft-ietf-rats-architecture`), versioned ID (`draft-ietf-rats-architecture-22`), canonical URL, or local file |
| `-o, --output <FILE>` | `<draft-name>.pdf` | Destination PDF file path |
| `--margin-right <MM>` | `38` | Custom right annotation margin width in mm |
| `--font-size <PT>` | `9.5` | Base body text font size in pt |
| `-h, --help` | | Print help information |
| `-V, --version` | | Print version |

---

## Legal & Trademarks

### Trademark Disclaimer
* **"Kindle"** and **"Kindle Scribe"** are registered trademarks of Amazon.com, Inc. or its affiliates.
* **"IETF"** and **"Internet-Draft"** are trademarks or service marks associated with the IETF Trust.
* `id2scribe` is an independent open-source project and is **not** affiliated with, endorsed by, sponsored by, or associated with Amazon.com, Inc. or the IETF Trust. Product names are referenced strictly for nominative purposes to describe hardware compatibility.

### Document Copyrights
`id2scribe` does not host, bundle, or redistribute any IETF Internet-Drafts or RFCs. All documents are retrieved on demand by the user from public IETF servers in accordance with the [IETF Trust Legal Provisions (TLP)](https://trustee.ietf.org/license-info/).

---

## License

Dual-licensed under either of:

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
