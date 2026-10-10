use headless_chrome::{Browser, LaunchOptionsBuilder};
use headless_chrome::types::PrintToPdfOptions;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Launching browser...");
    let launch_options = LaunchOptionsBuilder::default()
        .headless(true)
        .sandbox(false)
        .build()?;
    let browser = Browser::new(launch_options)?;
    let tab = browser.new_tab()?;
    
    let html_path = PathBuf::from("/Users/ionut/.gemini/antigravity/brain/42657571-9bc5-4a1e-8d3e-48cca5038498/scratch/injected_draft.html");
    let file_url = format!("file://{}", html_path.canonicalize()?.display());
    println!("Navigating to {}", file_url);
    tab.navigate_to(&file_url)?;
    tab.wait_until_navigated()?;

    println!("Printing to PDF...");
    let options = PrintToPdfOptions {
        landscape: Some(false),
        display_header_footer: Some(false),
        print_background: Some(true),
        scale: Some(1.0),
        paper_width: Some(158.0 / 25.4),
        paper_height: Some(210.0 / 25.4),
        margin_top: Some(0.0), // zero CDP margin so @page CSS rules take full effect
        margin_bottom: Some(0.0),
        margin_left: Some(0.0),
        margin_right: Some(0.0),
        prefer_css_page_size: Some(true),
        page_ranges: Some("12".to_string()), ..Default::default()
    };

    let pdf_data = tab.print_to_pdf(Some(options))?;
    let out_path = PathBuf::from("/Users/ionut/.gemini/antigravity/brain/42657571-9bc5-4a1e-8d3e-48cca5038498/scratch/page12.pdf");
    std::fs::write(&out_path, pdf_data)?;
    println!("Wrote {} bytes to {:?}", std::fs::metadata(&out_path)?.len(), out_path);
    Ok(())
}
