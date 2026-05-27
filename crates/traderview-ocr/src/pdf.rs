//! PDF text-layer extraction via lopdf.
//!
//! Born-digital PDFs (Amazon order receipts, Chase statements, vendor invoices)
//! carry a text layer. lopdf walks the content streams and yields the text
//! without needing to rasterize and OCR. Scanned PDFs (image-only) return an
//! empty string — caller treats that as `NeedsImage`.

use crate::OcrError;
use lopdf::Document;

pub fn extract_text(bytes: &[u8]) -> Result<String, OcrError> {
    let doc = Document::load_mem(bytes).map_err(|e| OcrError::Pdf(format!("load: {e}")))?;
    let mut out = String::new();
    // Page numbers are 1-indexed in lopdf.
    let page_count = doc.get_pages().len() as u32;
    for page_num in 1..=page_count {
        match doc.extract_text(&[page_num]) {
            Ok(text) => {
                out.push_str(&text);
                if !text.ends_with('\n') {
                    out.push('\n');
                }
            }
            Err(e) => {
                tracing::debug!(page = page_num, error = %e, "pdf page text extract failed");
                // Don't bail — some pages may not have text; we still want the others.
            }
        }
    }
    Ok(out)
}
