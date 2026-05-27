//! OCR engine wrapper around `pure-onnx-ocr-sync` (PaddleOCR DBNet + SVTR + line
//! orientation + document orientation via `tract-onnx`, pure Rust).
//!
//! Model layout under `model_dir` (all five required by the builder):
//!   det.onnx          — text detection (PaddleOCR DBNet, e.g. en_PP-OCRv4_det)
//!   rec.onnx          — text recognition (PaddleOCR SVTR, e.g. en_PP-OCRv4_rec)
//!   line_ori.onnx     — text-line orientation classifier (rotation correction per line)
//!   doc_ori.onnx      — document orientation classifier (full-page rotation)
//!   dict.txt          — character dictionary for the recognition model
//!
//! The two orientation models are exactly what we need for skewed phone-camera
//! receipts: PaddleOCR's full pipeline corrects rotation natively without us
//! having to add a Hough-transform pass.
//!
//! Models are NOT bundled — too large for a git checkout, and they change
//! between PaddleOCR releases. The desktop app downloads them on first OCR
//! call into `$APP_DATA_DIR/traderview/models/paddleocr/`, mirroring the
//! embedded-Postgres lazy-download pattern.
//!
//! Without the `engine` cargo feature, `run` returns `OcrError::EngineDisabled`
//! so the rest of the crate (PDF text extraction, parsing, matching) still
//! works.

use crate::OcrError;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct RawText {
    pub text: String,
    pub confidence: f32,
}

#[cfg(feature = "engine")]
pub fn run(bytes: &[u8], model_dir: &Path) -> Result<RawText, OcrError> {
    use image::ImageReader;

    let det = model_dir.join("det.onnx");
    let rec = model_dir.join("rec.onnx");
    let line_ori = model_dir.join("line_ori.onnx");
    let doc_ori = model_dir.join("doc_ori.onnx");
    let dict = model_dir.join("dict.txt");
    for required in [&det, &rec, &line_ori, &doc_ori, &dict] {
        if !required.exists() {
            return Err(OcrError::ModelsMissing {
                expected_dir: model_dir.display().to_string(),
            });
        }
    }

    let cursor = std::io::Cursor::new(bytes);
    let reader = ImageReader::new(cursor)
        .with_guessed_format()
        .map_err(|e| OcrError::Decode(e.to_string()))?;
    let img = reader
        .decode()
        .map_err(|e| OcrError::Decode(e.to_string()))?;

    let engine = pure_onnx_ocr_sync::OcrEngineBuilder::new()
        .det_model_path(&det)
        .rec_model_path(&rec)
        .text_line_ori_model_path(&line_ori)
        .doc_ori_model_path(&doc_ori)
        .dictionary_path(&dict)
        .build()
        .map_err(|e| OcrError::Engine(format!("build engine: {e}")))?;

    let regions = engine
        .run_from_image(&img)
        .map_err(|e| OcrError::Engine(format!("run: {e}")))?;

    // Sort by top-Y then left-X to recover natural reading order. The bounding
    // box is a geo_types Polygon<f64> — we take the min Y/X over its exterior.
    let mut sorted: Vec<_> = regions
        .into_iter()
        .map(|r| {
            let (min_x, min_y) = polygon_top_left(&r.bounding_box);
            (min_y, min_x, r)
        })
        .collect();
    sorted.sort_by(|a, b| {
        a.0.partial_cmp(&b.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    });

    let mut conf_sum = 0.0f32;
    let mut lines = Vec::with_capacity(sorted.len());
    for (_, _, r) in &sorted {
        conf_sum += r.confidence;
        lines.push(r.text.clone());
    }
    let confidence = if sorted.is_empty() {
        0.0
    } else {
        conf_sum / sorted.len() as f32
    };

    Ok(RawText {
        text: lines.join("\n"),
        confidence,
    })
}

#[cfg(feature = "engine")]
fn polygon_top_left(p: &pure_onnx_ocr_sync::Polygon<f64>) -> (f64, f64) {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    for c in p.exterior().coords() {
        if c.x < min_x {
            min_x = c.x;
        }
        if c.y < min_y {
            min_y = c.y;
        }
    }
    (min_x, min_y)
}

#[cfg(not(feature = "engine"))]
pub fn run(_bytes: &[u8], _model_dir: &Path) -> Result<RawText, OcrError> {
    Err(OcrError::EngineDisabled)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_text_holds_string_and_confidence() {
        // Smoke test on the struct — defends against accidental field
        // renames that would break engine→parse pipeline.
        let r = RawText {
            text: "TOTAL $12.34".into(),
            confidence: 0.85,
        };
        assert_eq!(r.text, "TOTAL $12.34");
        assert!((r.confidence - 0.85).abs() < 1e-6);
    }

    #[cfg(not(feature = "engine"))]
    #[test]
    fn run_returns_engine_disabled_when_feature_off() {
        // Default workspace build doesn't enable `engine` (heavy tract+ndarray
        // deps). The fallback must return a clear "this build doesn't include
        // OCR" error instead of pretending to OCR.
        let r = run(b"\xff\xd8\xff", Path::new("/nonexistent"));
        assert!(matches!(r, Err(OcrError::EngineDisabled)));
    }

    #[cfg(feature = "engine")]
    #[test]
    fn run_with_missing_models_returns_models_missing() {
        // When the engine feature IS on but the .onnx files aren't where
        // we expect, error must be ModelsMissing (so the UI can prompt
        // the user to download), not Engine (which implies a runtime bug).
        let r = run(b"\xff\xd8\xff", Path::new("/definitely/does/not/exist"));
        assert!(matches!(r, Err(OcrError::ModelsMissing { .. })));
    }
}
