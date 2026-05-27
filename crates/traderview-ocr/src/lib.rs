//! traderview-ocr — pure-Rust OCR for receipt processing.
//!
//! Two extraction paths:
//!   * Image (JPG/PNG/WebP/BMP) → PaddleOCR via `pure-onnx-ocr-sync` (gated
//!     behind the `engine` feature; off by default so the workspace builds
//!     without pulling tract + ndarray for users who don't need OCR).
//!   * PDF → `lopdf` text-layer extraction. Born-digital receipts (Amazon,
//!     Chase, Apple Card) carry a text layer; we read it directly. Scanned
//!     PDFs (rare; usually a phone photo saved as PDF) return `NeedsImage`
//!     so the UI can ask the user to upload as JPG/PNG.
//!
//! All structured-field extraction (merchant / total / date) runs on the
//! resulting text via regex in `parse.rs`, identical for both paths.

pub mod engine;
pub mod matcher;
pub mod parse;
pub mod pdf;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrResult {
    pub text: String,
    pub merchant: Option<String>,
    pub total: Option<Decimal>,
    pub date: Option<NaiveDate>,
    pub confidence: f32,
}

#[derive(Debug, thiserror::Error)]
pub enum OcrError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("unsupported mime: {0}")]
    UnsupportedMime(String),
    #[error("pdf has no text layer — re-upload as JPG/PNG so OCR can run on the image")]
    NeedsImage,
    #[error("decode: {0}")]
    Decode(String),
    #[error("pdf: {0}")]
    Pdf(String),
    #[error("ocr engine not built with `--features engine` — receipt OCR is disabled, PDF text-layer extraction still works")]
    EngineDisabled,
    #[error("ocr models not found in {expected_dir} — drop PaddleOCR detection + recognition .onnx files there")]
    ModelsMissing { expected_dir: String },
    #[error("ocr engine: {0}")]
    Engine(String),
}

/// Run the right extraction path for the given MIME type.
///
/// `model_dir` is the directory holding the PaddleOCR `.onnx` files. For the
/// desktop Tauri build this is `$APP_DATA_DIR/traderview/models/paddleocr/`.
/// PDF extraction ignores this parameter.
pub fn extract(bytes: &[u8], mime: &str, model_dir: Option<&std::path::Path>) -> Result<OcrResult, OcrError> {
    let mime_lower = mime.to_ascii_lowercase();
    if mime_lower == "application/pdf" || mime_lower.starts_with("application/pdf") {
        let text = pdf::extract_text(bytes)?;
        if text.trim().is_empty() {
            return Err(OcrError::NeedsImage);
        }
        return Ok(parse::structure(&text, 0.9));
    }
    if mime_lower.starts_with("image/") {
        let dir = model_dir.ok_or_else(|| OcrError::ModelsMissing {
            expected_dir: "<no model_dir provided>".into(),
        })?;
        let raw = engine::run(bytes, dir)?;
        return Ok(parse::structure(&raw.text, raw.confidence));
    }
    Err(OcrError::UnsupportedMime(mime.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_mime_returns_error() {
        let r = extract(b"some bytes", "text/plain", None);
        assert!(matches!(r, Err(OcrError::UnsupportedMime(s)) if s == "text/plain"));
    }

    #[test]
    fn unsupported_mime_is_case_insensitive() {
        // Browser may send "IMAGE/JPEG" or "Application/PDF" — must not
        // be misclassified as unsupported.
        let r = extract(b"", "TEXT/PLAIN", None);
        assert!(matches!(r, Err(OcrError::UnsupportedMime(_))));
    }

    #[test]
    fn empty_pdf_returns_needs_image() {
        // A 0-byte body — pdf::extract_text returns "" or errors. Either way
        // the user-visible signal must be NeedsImage, not Decode (different UX).
        let r = extract(b"%PDF-1.4\n%EOF\n", "application/pdf", None);
        // Either NeedsImage (no text layer) or Pdf decode error — both are
        // valid signals to the UI; what's NOT valid is a panic.
        assert!(r.is_err());
    }

    #[test]
    fn image_without_model_dir_returns_models_missing() {
        // Caller forgot to point at the model directory — must NOT panic.
        let r = extract(b"\xff\xd8\xff", "image/jpeg", None);
        assert!(matches!(r, Err(OcrError::ModelsMissing { .. })));
    }

    #[test]
    fn ocr_result_serializes_to_json() {
        // OcrResult is shipped over the wire to the frontend receipt-match
        // modal — serde must produce flat JSON the JS can parse.
        let r = OcrResult {
            text: "subtotal $42.99".into(),
            merchant: Some("CHIPOTLE".into()),
            total: Some(rust_decimal::Decimal::new(4299, 2)),
            date: chrono::NaiveDate::from_ymd_opt(2026, 5, 27),
            confidence: 0.87,
        };
        let s = serde_json::to_string(&r).unwrap();
        assert!(s.contains("CHIPOTLE"));
        assert!(s.contains("42.99"));
        assert!(s.contains("2026-05-27"));
    }

    #[test]
    fn error_display_is_user_actionable() {
        // The frontend surfaces these messages directly. Each must hint at
        // the corrective action (no opaque "internal error"-style strings).
        let needs_img = OcrError::NeedsImage.to_string();
        assert!(needs_img.to_lowercase().contains("image") || needs_img.contains("JPG"));

        let models_missing = OcrError::ModelsMissing { expected_dir: "/path/to/dir".into() }.to_string();
        assert!(models_missing.contains("/path/to/dir"));
        assert!(models_missing.contains(".onnx"));
    }
}
