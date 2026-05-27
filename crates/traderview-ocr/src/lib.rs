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
