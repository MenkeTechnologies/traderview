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

#[cfg(test)]
mod tests {
    use super::*;

    /// Empty input — lopdf rejects → Pdf error variant. The wizard
    /// surfaces this as "couldn't parse the PDF" rather than treating
    /// it as a text-less scan.
    #[test]
    fn empty_bytes_returns_pdf_error() {
        let r = extract_text(b"");
        assert!(matches!(r, Err(OcrError::Pdf(_))),
            "empty input should produce Pdf decode error, got: {:?}", r);
    }

    /// Garbage bytes (not even close to PDF) — Pdf error.
    #[test]
    fn random_bytes_return_pdf_error() {
        let r = extract_text(b"this is definitely not a PDF file");
        assert!(matches!(r, Err(OcrError::Pdf(_))));
    }

    /// File starting with `%PDF-` magic but otherwise corrupt — Pdf
    /// error. Defends against assuming the magic alone makes a doc
    /// parseable.
    #[test]
    fn fake_pdf_header_only_returns_error() {
        let r = extract_text(b"%PDF-1.4\nthis trailer is not valid\n%%EOF");
        assert!(matches!(r, Err(OcrError::Pdf(_))));
    }

    /// A real, minimal lopdf-constructed PDF with a single page of
    /// text — exercises the happy path. Uses lopdf to BUILD the
    /// fixture so the test doesn't depend on a hand-crafted byte
    /// stream that might break when lopdf updates.
    #[test]
    fn minimal_pdf_with_text_extracts_it() {
        use lopdf::content::{Content, Operation};
        use lopdf::{dictionary, Document, Object, Stream};

        let mut doc = Document::with_version("1.5");
        let pages_id = doc.new_object_id();
        let font_id = doc.add_object(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Courier",
        });
        let resources_id = doc.add_object(dictionary! {
            "Font" => dictionary! { "F1" => font_id },
        });
        let content = Content { operations: vec![
            Operation::new("BT", vec![]),
            Operation::new("Tf", vec!["F1".into(), 12.into()]),
            Operation::new("Td", vec![100.into(), 700.into()]),
            Operation::new("Tj", vec![Object::string_literal("Hello PDF World")]),
            Operation::new("ET", vec![]),
        ] };
        let content_id = doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "Contents" => content_id,
            "Resources" => resources_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
        });
        let pages = dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => 1,
        };
        doc.objects.insert(pages_id, Object::Dictionary(pages));
        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        doc.trailer.set("Root", catalog_id);
        let mut bytes = Vec::new();
        doc.save_to(&mut bytes).expect("save minimal pdf");

        let text = extract_text(&bytes).expect("extract should succeed");
        assert!(text.contains("Hello PDF World"),
            "extracted text must contain the embedded string, got: {:?}", text);
    }

    /// A valid PDF with NO text content (just an empty page) returns
    /// an empty or whitespace-only string. The caller in lib.rs uses
    /// this to trigger `OcrError::NeedsImage` ("re-upload as JPG").
    #[test]
    fn empty_pdf_page_yields_no_text() {
        use lopdf::{dictionary, Document, Object};

        let mut doc = Document::with_version("1.5");
        let pages_id = doc.new_object_id();
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 612.into(), 792.into()],
        });
        let pages = dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => 1,
        };
        doc.objects.insert(pages_id, Object::Dictionary(pages));
        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        doc.trailer.set("Root", catalog_id);
        let mut bytes = Vec::new();
        doc.save_to(&mut bytes).expect("save empty-page pdf");

        let text = extract_text(&bytes).expect("extract should succeed");
        // Empty or just a newline — caller treats either as NeedsImage.
        assert!(text.trim().is_empty(),
            "empty page should produce empty text, got: {:?}", text);
    }
}
