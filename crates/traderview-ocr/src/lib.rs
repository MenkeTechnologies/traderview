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
pub mod tax_forms;
pub mod matcher;
pub mod parse;
pub mod pdf;

use chrono::{NaiveDate, NaiveTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrResult {
    pub text: String,
    pub merchant: Option<String>,
    /// Street + city/state/zip — joined with a comma if both lines parsed.
    #[serde(default)]
    pub address: Option<String>,
    pub date: Option<NaiveDate>,
    /// Time-of-day from the receipt (e.g., `06:07PM` → `18:07:00`).
    #[serde(default)]
    pub time: Option<NaiveTime>,
    /// Pre-tax subtotal when separately printed.
    #[serde(default)]
    pub subtotal: Option<Decimal>,
    /// Tax amount when separately printed.
    #[serde(default)]
    pub tax: Option<Decimal>,
    /// Charged amount (post-tax). Equals `subtotal + tax` if all three parse.
    pub total: Option<Decimal>,
    /// Itemized line items with a best-guess category per item.
    #[serde(default)]
    pub items: Vec<OcrLineItem>,
    pub confidence: f32,
    /// Which backend produced this result: `"apple_vision"` /
    /// `"tesseract"` / `"pdf"` / `"unknown"`. Surfaced in the receipt
    /// modal so the user can see at a glance which engine ran, and
    /// targeted by bulk re-OCR jobs.
    #[serde(default = "default_engine")]
    pub engine: String,
}

fn default_engine() -> String {
    // Pre-engine-tracking JSONB rows deserialize with this default so
    // upgrades don't crash. Bulk re-OCR will overwrite as receipts get
    // re-processed.
    "unknown".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrLineItem {
    pub name: String,
    /// `None` when the receipt only shows the line total.
    #[serde(default)]
    pub qty: Option<Decimal>,
    #[serde(default)]
    pub unit_price: Option<Decimal>,
    pub line_total: Decimal,
    /// Best-guess Schedule C category id — see `parse::guess_category`
    /// for the canonical 20-bucket taxonomy.
    pub category: String,
    /// Tax bucket the item rolls up into. Auto-suggested from the
    /// category (`groceries → personal`, most others → `business`); the
    /// user can override per-item via the match modal. JSONB column on
    /// `receipts` stores the override so totals stay correct.
    ///
    /// Domain: `business` (Schedule C) | `rental` (Schedule E) |
    /// `personal` (non-deductible) | `unclassified` (no auto-default).
    #[serde(default = "default_unclassified_bucket")]
    pub tax_bucket: String,
    /// When `tax_bucket == "rental"`, links the item to the specific
    /// rental property the expense is allocated to (Schedule E
    /// allocates per-property). `None` for business / personal items.
    #[serde(default)]
    pub rental_property_id: Option<uuid::Uuid>,
}

fn default_unclassified_bucket() -> String {
    "unclassified".into()
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
pub fn extract(
    bytes: &[u8],
    mime: &str,
    model_dir: Option<&std::path::Path>,
) -> Result<OcrResult, OcrError> {
    let mime_lower = mime.to_ascii_lowercase();
    if mime_lower == "application/pdf" || mime_lower.starts_with("application/pdf") {
        let text = pdf::extract_text(bytes)?;
        if text.trim().is_empty() {
            return Err(OcrError::NeedsImage);
        }
        let mut result = parse::structure(&text, 0.9);
        result.engine = "pdf".into();
        return Ok(result);
    }
    if mime_lower.starts_with("image/") {
        let dir = model_dir.ok_or_else(|| OcrError::ModelsMissing {
            expected_dir: "<no model_dir provided>".into(),
        })?;
        // Run every backend the host can offer (Vision + Tesseract
        // psm4/psm6) and merge field-by-field. Picks Vision's clean
        // header for merchant/date and Tesseract's row alignment for
        // items; either engine's win on a particular field flows into
        // the final result.
        let backends = engine::run_all(bytes, dir);
        if backends.is_empty() {
            return Err(OcrError::Engine(
                "every OCR backend failed — check tesseract install \
                 and tools/tv-ocr-vision/build.sh"
                    .into(),
            ));
        }
        let parsed: Vec<OcrResult> = backends
            .into_iter()
            .map(|rt| {
                let mut r = parse::structure(&rt.text, rt.confidence);
                r.engine = rt.engine;
                r
            })
            .collect();
        return Ok(combine_results(parsed));
    }
    Err(OcrError::UnsupportedMime(mime.into()))
}

/// Field-level ensemble fusion of multiple `OcrResult`s produced from
/// running different OCR backends over the same image.
///
/// Strategy per field:
///   * Scalar fields (`merchant`, `date`, `total`, `subtotal`, `tax`,
///     `address`, `time`): walk results sorted by confidence DESC, use
///     the first `Some` value. Lets the highest-confidence engine
///     speak first, but a Tesseract win on a field Vision missed
///     still lands.
///   * `items`: union by `(lower(name), line_total)` so duplicates
///     from different engines collapse but unique finds from either
///     engine are kept.
///   * `text`: keep the highest-confidence engine's text (the parser
///     already ran over every variant, so this is just what surfaces
///     in the UI's raw-OCR diagnostics panel).
///   * `confidence`: max over all backends — a single high-quality
///     extraction shouldn't be diluted by a weaker backend's miss.
///   * `engine`: `ensemble:eng1+eng2+...` so the UI pill shows which
///     backends contributed and the bulk-reocr endpoint can filter on
///     `non_ensemble` to re-process old single-engine extractions.
fn combine_results(mut rs: Vec<OcrResult>) -> OcrResult {
    if rs.len() == 1 {
        return rs.pop().unwrap();
    }
    rs.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let engines: Vec<String> = rs.iter().map(|r| r.engine.clone()).collect();
    let mut acc = rs.remove(0);
    // Track max confidence across all backends.
    let mut max_conf = acc.confidence;
    for other in rs {
        acc.merchant = acc.merchant.or(other.merchant);
        acc.address = acc.address.or(other.address);
        acc.date = acc.date.or(other.date);
        acc.time = acc.time.or(other.time);
        acc.subtotal = acc.subtotal.or(other.subtotal);
        acc.tax = acc.tax.or(other.tax);
        acc.total = acc.total.or(other.total);
        for item in other.items {
            let dup = acc.items.iter().any(|p| {
                p.name.to_lowercase() == item.name.to_lowercase()
                    && p.line_total == item.line_total
            });
            if !dup {
                acc.items.push(item);
            }
        }
        if other.confidence > max_conf {
            max_conf = other.confidence;
        }
    }
    acc.confidence = max_conf;
    acc.engine = format!("ensemble:{}", engines.join("+"));
    acc
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
    fn extract_application_pdf_with_charset_suffix_routes_to_pdf() {
        // Some upstream tools emit "application/pdf; charset=..." even
        // though PDF isn't text. The MIME prefix match must still route.
        let r = extract(b"", "application/pdf; charset=binary", None);
        // Empty bytes → either NeedsImage (no text layer) or Pdf decode
        // error — both are PDF-path outcomes, NOT UnsupportedMime.
        assert!(
            !matches!(r, Err(OcrError::UnsupportedMime(_))),
            "PDF with parameters must route to PDF path, got: {:?}", r,
        );
    }

    #[test]
    fn extract_uppercase_image_mime_routes_to_image_path() {
        // Browser uploads sometimes uppercase the MIME — must not
        // misclassify as unsupported.
        let r = extract(b"\xff\xd8\xff", "IMAGE/JPEG", None);
        // Without model_dir → ModelsMissing (the image-path failure mode),
        // NOT UnsupportedMime.
        assert!(matches!(r, Err(OcrError::ModelsMissing { .. })),
            "uppercase image/* MIME must reach image path, got: {:?}", r);
    }

    #[test]
    fn extract_image_webp_takes_image_path() {
        // image/* is a prefix match — image/webp, image/heic must all
        // route through the image path.
        let r = extract(b"\x00\x00", "image/webp", None);
        assert!(matches!(r, Err(OcrError::ModelsMissing { .. })));
        let r = extract(b"\x00\x00", "image/heic", None);
        assert!(matches!(r, Err(OcrError::ModelsMissing { .. })));
    }

    #[test]
    fn extract_unknown_mime_is_unsupported() {
        let r = extract(b"data", "video/mp4", None);
        assert!(matches!(r, Err(OcrError::UnsupportedMime(s)) if s == "video/mp4"));
    }

    #[test]
    fn vision_sidecar_json_roundtrips_through_inner_struct() {
        // Pin the shape the engine module's private VisionOut/VisionLine
        // structs accept — using a clone of the same shape here as a
        // public contract test. If the Swift sidecar changes its JSON
        // output, this catches the divergence.
        #[derive(serde::Deserialize)]
        struct VL { text: String, #[serde(default)] confidence: f32 }
        #[derive(serde::Deserialize)]
        struct VO {
            #[serde(default)] lines: Vec<VL>,
            #[serde(default)] confidence_mean: f32,
        }

        // Full shape — Swift sidecar emits all fields on a normal scan.
        let full = r#"{
            "engine": "apple_vision",
            "lines": [{"text": "TOTAL", "confidence": 0.99, "bbox": [0.1, 0.2, 0.3, 0.4]}],
            "line_count": 1,
            "confidence_mean": 0.99,
            "confidence_min": 0.99
        }"#;
        let parsed: VO = serde_json::from_str(full).expect("full json must parse");
        assert_eq!(parsed.lines.len(), 1);
        assert_eq!(parsed.lines[0].text, "TOTAL");
        assert!((parsed.confidence_mean - 0.99).abs() < 1e-3);

        // Minimal shape — Vision returns an empty image with no text.
        // No `lines` or `confidence_mean` fields. Defaults must apply.
        let minimal = r#"{"engine": "apple_vision", "line_count": 0}"#;
        let parsed: VO = serde_json::from_str(minimal).expect("minimal json must parse");
        assert_eq!(parsed.lines.len(), 0);
        assert_eq!(parsed.confidence_mean, 0.0);

        // Future extension — a new field the sidecar adds in v2 must
        // NOT break the Rust parser (forward compat).
        let with_extras = r#"{
            "engine": "apple_vision",
            "lines": [],
            "future_field": "ignored",
            "another": {"nested": true}
        }"#;
        let parsed: VO = serde_json::from_str(with_extras).expect("forward-compat parse");
        assert_eq!(parsed.lines.len(), 0);
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
            address: None,
            date: chrono::NaiveDate::from_ymd_opt(2026, 5, 27),
            time: None,
            subtotal: None,
            tax: None,
            total: Some(rust_decimal::Decimal::new(4299, 2)),
            items: Vec::new(),
            confidence: 0.87,
            engine: "apple_vision".into(),
        };
        let s = serde_json::to_string(&r).unwrap();
        assert!(s.contains("CHIPOTLE"));
        assert!(s.contains("42.99"));
        assert!(s.contains("2026-05-27"));
        // Engine tag is part of the wire contract so the UI can label
        // which backend produced this result.
        assert!(s.contains("apple_vision"));
    }

    #[test]
    fn combine_results_picks_highest_confidence_scalars() {
        // Two backends — one finds merchant + total but no date, the
        // other finds date + total (different value). Highest
        // confidence wins on total; missing fields fall through to
        // the other backend.
        let a = OcrResult {
            text: "vision raw".into(),
            merchant: Some("CHIPOTLE".into()),
            address: None,
            date: None,
            time: None,
            subtotal: None,
            tax: None,
            total: Some(rust_decimal::Decimal::new(4299, 2)),  // 42.99
            items: vec![OcrLineItem {
                name: "Burrito".into(), qty: None, unit_price: None,
                line_total: rust_decimal::Decimal::new(1099, 2),
                category: "meals".into(), tax_bucket: "business".into(),
                rental_property_id: None,
            }],
            confidence: 0.95,  // higher
            engine: "apple_vision".into(),
        };
        let b = OcrResult {
            text: "tesseract raw".into(),
            merchant: None,
            address: None,
            date: chrono::NaiveDate::from_ymd_opt(2026, 5, 27),
            time: None,
            subtotal: None,
            tax: None,
            total: Some(rust_decimal::Decimal::new(4250, 2)),  // 42.50, lower conf
            items: vec![OcrLineItem {
                name: "Chips & Salsa".into(), qty: None, unit_price: None,
                line_total: rust_decimal::Decimal::new(395, 2),
                category: "meals".into(), tax_bucket: "business".into(),
                rental_property_id: None,
            }],
            confidence: 0.85,
            engine: "tesseract_psm4".into(),
        };
        let m = super::combine_results(vec![a, b]);
        // High-conf wins on total.
        assert_eq!(m.total, Some(rust_decimal::Decimal::new(4299, 2)));
        // Low-conf provides the date (high-conf had None).
        assert!(m.date.is_some());
        // Items unioned — distinct names → both present.
        assert_eq!(m.items.len(), 2);
        // Merchant comes from high-conf.
        assert_eq!(m.merchant.as_deref(), Some("CHIPOTLE"));
        // Engine tag carries both backend names.
        assert!(m.engine.starts_with("ensemble:"));
        assert!(m.engine.contains("apple_vision"));
        assert!(m.engine.contains("tesseract_psm4"));
        // Confidence is the max.
        assert!((m.confidence - 0.95).abs() < 1e-6);
    }

    #[test]
    fn combine_results_dedupes_items_by_name_and_total() {
        // Same item parsed by both engines — must collapse to one row.
        let item = OcrLineItem {
            name: "Burrito".into(), qty: None, unit_price: None,
            line_total: rust_decimal::Decimal::new(1099, 2),
            category: "meals".into(), tax_bucket: "business".into(),
            rental_property_id: None,
        };
        let a = OcrResult {
            text: "a".into(), merchant: None, address: None, date: None,
            time: None, subtotal: None, tax: None, total: None,
            items: vec![item.clone()],
            confidence: 0.95, engine: "apple_vision".into(),
        };
        let b = OcrResult {
            text: "b".into(), merchant: None, address: None, date: None,
            time: None, subtotal: None, tax: None, total: None,
            // Same total + same name (case-insensitive) → duplicate.
            items: vec![OcrLineItem { name: "BURRITO".into(), ..item }],
            confidence: 0.85, engine: "tesseract_psm4".into(),
        };
        let m = super::combine_results(vec![a, b]);
        assert_eq!(m.items.len(), 1, "duplicate item should be deduped");
    }

    #[test]
    fn combine_results_single_backend_passes_through() {
        // Single-backend input — must round-trip unchanged.
        let r = OcrResult {
            text: "raw".into(),
            merchant: Some("M".into()),
            address: None, date: None, time: None,
            subtotal: None, tax: None, total: None,
            items: Vec::new(), confidence: 0.7,
            engine: "tesseract_psm4".into(),
        };
        let m = super::combine_results(vec![r.clone()]);
        assert_eq!(m.engine, r.engine);  // NOT prefixed with "ensemble:" when single
        assert_eq!(m.merchant, r.merchant);
    }

    #[test]
    fn combine_results_three_backend_ensemble_aggregates_engines() {
        // 3 backends representing the Vision+Tesseract×2 production
        // ensemble. Each gets a different field right; merger should
        // pull the best from each and tag all three engines in the
        // result label.
        let vision = OcrResult {
            text: "vision".into(),
            merchant: Some("CHIPOTLE".into()),  // vision wins (highest conf)
            address: None,
            date: None,                          // missing
            time: None,
            subtotal: None,
            tax: None,
            total: Some(rust_decimal::Decimal::new(4299, 2)),
            items: Vec::new(),
            confidence: 0.95,
            engine: "apple_vision".into(),
        };
        let tess4 = OcrResult {
            text: "tess4".into(),
            merchant: None,
            address: Some("123 Main St".into()), // tess4 contributes address
            date: chrono::NaiveDate::from_ymd_opt(2026, 5, 27),
            time: None,
            subtotal: None,
            tax: None,
            total: None,
            items: Vec::new(),
            confidence: 0.85,
            engine: "tesseract_psm4".into(),
        };
        let tess6 = OcrResult {
            text: "tess6".into(),
            merchant: None,
            address: None,
            date: None,
            time: chrono::NaiveTime::from_hms_opt(18, 30, 0),  // tess6 contributes time
            subtotal: None,
            tax: None,
            total: None,
            items: Vec::new(),
            confidence: 0.75,
            engine: "tesseract_psm6".into(),
        };
        let m = super::combine_results(vec![vision, tess4, tess6]);
        // High-conf vision wins merchant + total.
        assert_eq!(m.merchant.as_deref(), Some("CHIPOTLE"));
        assert_eq!(m.total, Some(rust_decimal::Decimal::new(4299, 2)));
        // Lower-conf backends contribute fields the higher-conf missed.
        assert_eq!(m.address.as_deref(), Some("123 Main St"));
        assert!(m.date.is_some());
        assert!(m.time.is_some());
        // Engine label carries all three backends in confidence-sorted order.
        assert_eq!(m.engine, "ensemble:apple_vision+tesseract_psm4+tesseract_psm6");
        // Confidence = max.
        assert_eq!(m.confidence, 0.95);
    }

    #[test]
    fn combine_results_handles_two_empty_backends() {
        // Both backends returned nothing useful (very low quality
        // image). Merger should not panic and should produce a result
        // whose engine label still tags both.
        let mk = |engine: &str, conf: f32| OcrResult {
            text: String::new(),
            merchant: None, address: None,
            date: None, time: None,
            subtotal: None, tax: None, total: None,
            items: Vec::new(),
            confidence: conf,
            engine: engine.to_string(),
        };
        let m = super::combine_results(vec![mk("apple_vision", 0.5), mk("tesseract_psm4", 0.4)]);
        assert!(m.engine.starts_with("ensemble:"));
        assert!(m.merchant.is_none());
        assert!(m.items.is_empty());
    }

    #[test]
    fn combine_results_items_partial_overlap_unions_correctly() {
        // Vision finds 3 items; Tesseract finds 2 (one overlap, one
        // unique). Final items = 4 (union).
        let mk_item = |name: &str, total: i64| OcrLineItem {
            name: name.into(), qty: None, unit_price: None,
            line_total: rust_decimal::Decimal::from(total),
            category: "meals".into(), tax_bucket: "business".into(),
            rental_property_id: None,
        };
        let vision = OcrResult {
            text: "v".into(), merchant: None, address: None,
            date: None, time: None, subtotal: None, tax: None, total: None,
            items: vec![mk_item("Burrito", 12), mk_item("Chips", 4), mk_item("Drink", 3)],
            confidence: 0.9, engine: "apple_vision".into(),
        };
        let tess = OcrResult {
            text: "t".into(), merchant: None, address: None,
            date: None, time: None, subtotal: None, tax: None, total: None,
            // "Burrito" duplicates Vision's; "Tip" is unique to Tesseract.
            items: vec![mk_item("Burrito", 12), mk_item("Tip", 2)],
            confidence: 0.8, engine: "tesseract_psm4".into(),
        };
        let m = super::combine_results(vec![vision, tess]);
        assert_eq!(m.items.len(), 4,
            "expected 4 unique items, got: {:?}",
            m.items.iter().map(|i| &i.name).collect::<Vec<_>>());
        // Ordering preserved: vision items first (highest conf), then
        // unique-to-tesseract entries appended.
        assert_eq!(m.items[0].name, "Burrito");
        assert!(m.items.iter().any(|i| i.name == "Tip"),
            "tess-unique 'Tip' must survive the union");
    }

    #[test]
    fn combine_results_items_same_name_different_total_kept_separately() {
        // Two backends saw the same merchant but caught DIFFERENT
        // totals on a same-named row (OCR ambiguity on a smudged
        // digit). The merger keys on (name, total) — so both rows
        // survive, which is the safe behavior: the user reviews
        // both, deletes the wrong one.
        let mk_item = |name: &str, total: i64| OcrLineItem {
            name: name.into(), qty: None, unit_price: None,
            line_total: rust_decimal::Decimal::from(total),
            category: "meals".into(), tax_bucket: "business".into(),
            rental_property_id: None,
        };
        let a = OcrResult {
            text: "a".into(), merchant: None, address: None,
            date: None, time: None, subtotal: None, tax: None, total: None,
            items: vec![mk_item("Burrito", 12)],
            confidence: 0.9, engine: "apple_vision".into(),
        };
        let b = OcrResult {
            text: "b".into(), merchant: None, address: None,
            date: None, time: None, subtotal: None, tax: None, total: None,
            items: vec![mk_item("Burrito", 13)],  // smudged 2 → 3
            confidence: 0.8, engine: "tesseract_psm4".into(),
        };
        let m = super::combine_results(vec![a, b]);
        assert_eq!(m.items.len(), 2,
            "different totals must NOT be deduped — user reviews both");
    }

    #[test]
    fn error_display_is_user_actionable() {
        // The frontend surfaces these messages directly. Each must hint at
        // the corrective action (no opaque "internal error"-style strings).
        let needs_img = OcrError::NeedsImage.to_string();
        assert!(needs_img.to_lowercase().contains("image") || needs_img.contains("JPG"));

        let models_missing = OcrError::ModelsMissing {
            expected_dir: "/path/to/dir".into(),
        }
        .to_string();
        assert!(models_missing.contains("/path/to/dir"));
        assert!(models_missing.contains(".onnx"));
    }

    #[test]
    fn error_unsupported_mime_includes_offending_mime() {
        // Frontend surfaces the bad MIME so user knows what they uploaded.
        let s = OcrError::UnsupportedMime("text/plain".into()).to_string();
        assert!(s.contains("text/plain"),
            "UnsupportedMime must echo the bad MIME, got: {s}");
    }

    #[test]
    fn error_decode_includes_inner_cause() {
        let s = OcrError::Decode("corrupt JPEG".into()).to_string();
        assert!(s.contains("corrupt JPEG"));
        assert!(s.to_lowercase().contains("decode"));
    }

    #[test]
    fn error_pdf_includes_inner_cause() {
        let s = OcrError::Pdf("malformed object stream".into()).to_string();
        assert!(s.contains("malformed object stream"));
        assert!(s.to_lowercase().contains("pdf"));
    }

    #[test]
    fn error_engine_includes_inner_cause() {
        let s = OcrError::Engine("tesseract exit 1".into()).to_string();
        assert!(s.contains("tesseract exit 1"));
    }

    #[test]
    fn error_engine_disabled_mentions_feature_flag() {
        // User builds without `--features engine`/`paddle` and gets a
        // useful error pointing at the build flag.
        let s = OcrError::EngineDisabled.to_string();
        assert!(s.contains("--features"),
            "EngineDisabled should hint at the feature flag");
    }

    #[test]
    fn error_io_from_std_io_error() {
        // The `#[from] std::io::Error` derive needs the conversion to
        // produce a usable Display. Wrap a real io::Error and confirm.
        let inner = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let err: OcrError = inner.into();
        let s = err.to_string();
        assert!(s.contains("io"));
        assert!(s.contains("missing"));
    }
}
