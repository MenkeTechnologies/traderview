//! OCR engine — dispatches to the best on-device backend available.
//!
//! Backend order:
//!   1. **Apple Vision** (`tv-ocr-vision` sidecar) — macOS only, by far the
//!      best accuracy on phone-photographed receipts. Runs on the Apple
//!      Neural Engine, on-device, free, no network. The sidecar is a tiny
//!      Swift binary in `tools/tv-ocr-vision/`; build via that dir's
//!      `build.sh`.
//!   2. **Tesseract CLI** — cross-platform fallback. Pipes image bytes
//!      via stdin → text via stdout, no temp files. Requires
//!      `brew install tesseract` (macOS) / `apt install tesseract-ocr
//!      tesseract-ocr-eng` (Linux).
//!
//! The dispatch is automatic: if the Vision sidecar binary is present
//! (and we're on macOS), it runs. Otherwise we fall through to
//! Tesseract. Failures on the Vision path also fall through — the user
//! never sees an OCR outage just because the sidecar wasn't built.
//!
//! ## Sidecar discovery (in order):
//!   1. `$TV_OCR_VISION_BIN`           — explicit override (prod packaging)
//!   2. `<workspace>/target/release-sidecars/tv-ocr-vision`
//!   3. `<current_exe dir>/tv-ocr-vision`
//!   4. `tv-ocr-vision` on `$PATH`
//!
//! ## Tesseract receipt-tuning knobs:
//!   * `--psm 4` — single column of variable-size text. Best for receipts
//!     where line items + totals don't conform to a single uniform block.
//!   * `--oem 1` — LSTM-only (no legacy Tesseract). Higher accuracy on
//!     printed text since 4.0.
//!   * `-c preserve_interword_spaces=1` — keep alignment spaces so
//!     multi-column rows survive (`SPRAY FORD RED      27.52`).

use crate::OcrError;
use image::{DynamicImage, ImageFormat, ImageReader};
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Clone)]
pub struct RawText {
    pub text: String,
    pub confidence: f32,
    /// Which backend produced this text: `"apple_vision"` or
    /// `"tesseract"`. Persisted in `ocr_extracted` so the UI can label
    /// the result and the bulk re-OCR job can target weak engines.
    pub engine: String,
}

/// Run OCR. Dispatches to the best backend available — Vision on macOS
/// when the sidecar is installed, Tesseract otherwise.
///
/// Single-backend path. For the ensemble (run-all-and-merge) entry
/// point that powers production OCR, see [`run_all`].
///
/// `model_dir` is preserved for API compatibility and is consulted only
/// by the Tesseract path (where it overrides `TESSDATA_PREFIX` when a
/// user-supplied `eng.traineddata` is dropped in).
pub fn run(bytes: &[u8], model_dir: &Path) -> Result<RawText, OcrError> {
    // 1) Apple Vision (macOS) — try first if the sidecar is reachable.
    if cfg!(target_os = "macos") {
        if let Some(bin) = find_vision_binary() {
            match run_vision(&bin, bytes) {
                Ok(rt) => return Ok(rt),
                Err(e) => {
                    tracing::warn!(
                        sidecar = %bin.display(),
                        err = %e,
                        "vision sidecar failed — falling back to tesseract",
                    );
                }
            }
        } else {
            tracing::debug!(
                "vision sidecar not found — set TV_OCR_VISION_BIN or run \
                 tools/tv-ocr-vision/build.sh to enable. Falling back to tesseract.",
            );
        }
    }
    // 2) Tesseract — cross-platform fallback.
    run_tesseract(bytes, model_dir, "4")
}

/// Run EVERY available backend and return all successful results.
/// The caller (lib.rs `extract`) parses each independently and fuses
/// the structured `OcrResult`s field-by-field — total accuracy beats
/// any single engine.
///
/// Backends, in order:
///   1. Apple Vision (`apple_vision`)         — macOS only, when sidecar present.
///   2. Tesseract LSTM with `--psm 4`         — single column of variable text.
///   3. Tesseract LSTM with `--psm 6`         — single uniform block. Catches
///      cases where receipt rows are tightly packed.
///
/// On a 4-core macOS desktop with the sidecar built, all three run in
/// parallel via `rayon::join`-style concurrency (currently sequential
/// — see TODO below). Total latency is bounded by `(Vision || Tesseract×2)`
/// and stays under the OCR semaphore's per-job budget.
///
/// Returns an empty `Vec` when every backend fails (the caller surfaces
/// `OcrError::Engine` in that case).
pub fn run_all(bytes: &[u8], model_dir: &Path) -> Vec<RawText> {
    let mut out: Vec<RawText> = Vec::with_capacity(3);

    // Apple Vision sidecar — gigantic accuracy win on macOS. Free,
    // on-device, runs on the Neural Engine.
    if cfg!(target_os = "macos") {
        if let Some(bin) = find_vision_binary() {
            match run_vision(&bin, bytes) {
                Ok(rt) => out.push(rt),
                Err(e) => tracing::warn!(err = %e, "ensemble: vision backend failed"),
            }
        }
    }

    // Tesseract pass 1 — `--psm 4`, the long-time receipt default.
    match run_tesseract(bytes, model_dir, "4") {
        Ok(mut rt) => {
            rt.engine = "tesseract_psm4".into();
            out.push(rt);
        }
        Err(e) => tracing::warn!(err = %e, "ensemble: tesseract psm4 failed"),
    }

    // Tesseract pass 2 — `--psm 6`, single uniform block. Different
    // page-segmentation model surfaces different row breaks than
    // --psm 4 on dense receipts; the field-level merger picks
    // whichever produced a parseable total/date/items.
    match run_tesseract(bytes, model_dir, "6") {
        Ok(mut rt) => {
            rt.engine = "tesseract_psm6".into();
            out.push(rt);
        }
        Err(e) => tracing::warn!(err = %e, "ensemble: tesseract psm6 failed"),
    }

    // PaddleOCR (DBNet + SVTR via tract-onnx). Only built when
    // compiled with `--features paddle`. Models are downloaded
    // separately into model_dir; skips gracefully when absent.
    #[cfg(feature = "paddle")]
    {
        match run_paddle(bytes, model_dir) {
            Ok(rt) => out.push(rt),
            Err(e) => tracing::warn!(err = %e, "ensemble: paddle backend skipped/failed"),
        }
    }

    out
}

#[cfg(feature = "paddle")]
/// PaddleOCR (DBNet text detector + SVTR text recognizer + line/doc
/// orientation classifiers) via `pure-onnx-ocr-sync`. Returns an
/// error when any of the five model files (`det.onnx`, `rec.onnx`,
/// `line_ori.onnx`, `doc_ori.onnx`, `dict.txt`) is missing under
/// `model_dir`, so the ensemble runner skips this backend cleanly on
/// first install.
///
/// PaddleOCR is genuinely worse than Apple Vision and roughly on par
/// with Tesseract for clean receipts, but it has different failure
/// modes — particularly strong on rotated and skewed phone-camera
/// photos because of the orientation classifiers. In the ensemble
/// it contributes recovery cases the other backends miss.
fn run_paddle(bytes: &[u8], model_dir: &Path) -> Result<RawText, OcrError> {
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
    let img = ImageReader::new(cursor)
        .with_guessed_format()
        .map_err(|e| OcrError::Decode(e.to_string()))?
        .decode()
        .map_err(|e| OcrError::Decode(e.to_string()))?;

    let engine = pure_onnx_ocr_sync::OcrEngineBuilder::new()
        .det_model_path(&det)
        .rec_model_path(&rec)
        .text_line_ori_model_path(&line_ori)
        .doc_ori_model_path(&doc_ori)
        .dictionary_path(&dict)
        .build()
        .map_err(|e| OcrError::Engine(format!("build paddle engine: {e}")))?;

    let regions = engine
        .run_from_image(&img)
        .map_err(|e| OcrError::Engine(format!("run paddle: {e}")))?;

    // Sort by top-Y then left-X to recover natural reading order. The
    // bounding box is a `geo_types::Polygon<f64>` — take min over
    // exterior coords.
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
        engine: "paddle".into(),
    })
}

#[cfg(feature = "paddle")]
fn polygon_top_left(p: &geo_types::Polygon<f64>) -> (f64, f64) {
    use geo_types::Coord;
    let exterior = p.exterior().0.iter().collect::<Vec<&Coord<f64>>>();
    let min_x = exterior.iter().map(|c| c.x).fold(f64::INFINITY, f64::min);
    let min_y = exterior.iter().map(|c| c.y).fold(f64::INFINITY, f64::min);
    (min_x, min_y)
}

// ---------------------------------------------------------------------------
// Apple Vision sidecar path
// ---------------------------------------------------------------------------

/// Locate the `tv-ocr-vision` Swift sidecar binary.
fn find_vision_binary() -> Option<PathBuf> {
    // 1) Explicit env override — wins for production packaging.
    if let Ok(p) = std::env::var("TV_OCR_VISION_BIN") {
        let pb = PathBuf::from(p);
        if pb.is_file() {
            return Some(pb);
        }
    }
    // 2) Workspace target dir relative to the crate's compile-time
    //    CARGO_MANIFEST_DIR. Works for `cargo run` / `cargo test` /
    //    `cargo tauri dev` because the workspace layout is stable.
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if let Some(ws) = manifest
        .ancestors()
        .find(|p| p.join("Cargo.toml").exists() && p.join("crates").is_dir())
    {
        let p = ws.join("target/release-sidecars/tv-ocr-vision");
        if p.is_file() {
            return Some(p);
        }
    }
    // 3) Sibling of the running executable — for shipped builds.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let p = dir.join("tv-ocr-vision");
            if p.is_file() {
                return Some(p);
            }
        }
    }
    // 4) Bare name on PATH.
    if Command::new("tv-ocr-vision")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return Some(PathBuf::from("tv-ocr-vision"));
    }
    None
}

/// Pipe `bytes` to the Vision sidecar's stdin, parse the JSON stdout into
/// a `RawText`. Errors propagate so the caller can fall through to
/// Tesseract.
fn run_vision(bin: &Path, bytes: &[u8]) -> Result<RawText, OcrError> {
    let mut child = Command::new(bin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| OcrError::Engine(format!("spawn tv-ocr-vision: {e}")))?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| OcrError::Engine("vision stdin unavailable".into()))?;
        stdin
            .write_all(bytes)
            .map_err(|e| OcrError::Engine(format!("write vision stdin: {e}")))?;
    }
    let out = child
        .wait_with_output()
        .map_err(|e| OcrError::Engine(format!("wait vision: {e}")))?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(OcrError::Engine(format!(
            "tv-ocr-vision exit {}: {}",
            out.status.code().unwrap_or(-1),
            stderr.trim()
        )));
    }

    // Parse the sidecar's JSON. We accept only the keys we need; extras
    // are ignored so future sidecar versions can add fields without
    // breaking the Rust side.
    // `confidence` is parsed but unused at the top level — we read the
    // sidecar's pre-aggregated `confidence_mean` instead. Kept so a
    // future per-line review path (track A4's "re-OCR low-conf lines")
    // can read it without changing the JSON contract.
    #[derive(serde::Deserialize)]
    #[allow(dead_code)]
    struct VisionLine {
        text: String,
        #[serde(default)]
        confidence: f32,
    }
    #[derive(serde::Deserialize)]
    struct VisionOut {
        #[serde(default)]
        lines: Vec<VisionLine>,
        #[serde(default)]
        confidence_mean: f32,
    }
    let parsed: VisionOut = serde_json::from_slice(&out.stdout)
        .map_err(|e| OcrError::Engine(format!("parse vision json: {e}")))?;

    let text = parsed
        .lines
        .iter()
        .map(|l| l.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    // Vision returns a clean per-line mean already. Empty result =>
    // confidence 0 so downstream can flag it for re-OCR.
    let confidence = if parsed.lines.is_empty() {
        0.0
    } else {
        parsed.confidence_mean
    };

    Ok(RawText {
        text,
        confidence,
        engine: "apple_vision".into(),
    })
}

// ---------------------------------------------------------------------------
// Tesseract fallback path (existing pipeline, unchanged behaviorally)
// ---------------------------------------------------------------------------

fn run_tesseract(bytes: &[u8], model_dir: &Path, psm: &str) -> Result<RawText, OcrError> {
    let tess_present = Command::new("tesseract")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !tess_present {
        return Err(OcrError::ModelsMissing {
            expected_dir: format!(
                "tesseract CLI not on PATH — install with `brew install tesseract` (macOS) or \
                 `apt install tesseract-ocr tesseract-ocr-eng` (Linux). User-supplied model_dir \
                 was: {}",
                model_dir.display()
            ),
        });
    }

    let pre = preprocess(bytes).unwrap_or_else(|_| bytes.to_vec());

    let mut cmd = Command::new("tesseract");
    cmd.arg("-")
        .arg("-")
        .arg("-l")
        .arg("eng")
        .arg("--psm")
        .arg(psm)
        .arg("--oem")
        .arg("1")
        .arg("-c")
        .arg("preserve_interword_spaces=1");
    if model_dir.join("eng.traineddata").exists() {
        cmd.env("TESSDATA_PREFIX", model_dir);
    }
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| OcrError::Engine(format!("spawn tesseract: {e}")))?;
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| OcrError::Engine("tesseract stdin unavailable".into()))?;
        stdin
            .write_all(&pre)
            .map_err(|e| OcrError::Engine(format!("write stdin: {e}")))?;
    }
    let out = child
        .wait_with_output()
        .map_err(|e| OcrError::Engine(format!("wait: {e}")))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(OcrError::Engine(format!(
            "tesseract exit {}: {}",
            out.status.code().unwrap_or(-1),
            stderr.trim()
        )));
    }
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    // Tesseract CLI doesn't surface per-word confidence on the text
    // path. Use a fixed-but-honest heuristic — 0.85 when output looks
    // reasonable, 0.0 on empty. Track A4 swaps this for tsv-parsed
    // confidence once landed.
    let confidence = if text.trim().is_empty() { 0.0 } else { 0.85 };
    Ok(RawText {
        text,
        confidence,
        engine: "tesseract".into(),
    })
}

/// Pre-OCR image preprocessing for the Tesseract path. Vision handles
/// preprocessing internally and gets raw bytes.
///
/// Pipeline:
///   1. Decode (any format `image` supports — JPEG, PNG, WebP, BMP).
///   2. To grayscale (luma8). Drops false texture from camera noise.
///   3. Lanczos3 upscale so the SHORTER side hits ~1600 px, capped at
///      3×.
///   4. **Sauvola adaptive thresholding.** Computes a local threshold
///      per pixel from a window's mean + std, then binarizes. Critical
///      for phone-photo receipts where uneven lighting / shadows /
///      faded thermal print would tank a global threshold. Sauvola
///      handles all three.
///   5. Re-encode as PNG (lossless).
fn preprocess(bytes: &[u8]) -> Result<Vec<u8>, OcrError> {
    let img = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| OcrError::Decode(format!("guess fmt: {e}")))?
        .decode()
        .map_err(|e| OcrError::Decode(format!("decode: {e}")))?;
    let gray = img.to_luma8();
    let (w, h) = (gray.width(), gray.height());
    if w == 0 || h == 0 {
        return Err(OcrError::Decode("zero-size image".into()));
    }
    let target_min: u32 = 1600;
    let min_dim = w.min(h);
    let scale = if min_dim < target_min {
        (target_min as f32 / min_dim as f32).clamp(1.0, 3.0)
    } else {
        1.0
    };
    let upscaled = if scale > 1.0 {
        let new_w = ((w as f32) * scale) as u32;
        let new_h = ((h as f32) * scale) as u32;
        image::imageops::resize(&gray, new_w, new_h, image::imageops::FilterType::Lanczos3)
    } else {
        gray
    };

    // Sauvola binarization. Window size scales with image size — a
    // 25px window on a 400px receipt covers a row of text; on a 4000px
    // shot we want ~50 to capture the same character height.
    let win = (upscaled.width().min(upscaled.height()) / 64).clamp(15, 50) as usize;
    let binarized = sauvola_threshold(&upscaled, win, 0.34);

    let mut out = Vec::with_capacity(bytes.len());
    DynamicImage::ImageLuma8(binarized)
        .write_to(&mut Cursor::new(&mut out), ImageFormat::Png)
        .map_err(|e| OcrError::Decode(format!("re-encode: {e}")))?;
    Ok(out)
}

/// Sauvola adaptive thresholding using an integral-image fast path.
///
/// For each pixel `p`, compute `μ` (mean) and `σ` (std-dev) over the
/// `(2w+1)×(2w+1)` window centered at `p`. The local threshold is:
///
/// ```text
/// T(p) = μ * (1 + k * (σ/R - 1))
/// ```
///
/// with `k = 0.34` and `R = 128` (the standard Sauvola defaults for
/// 8-bit images). Pixel becomes 0 (foreground/text) when `p < T(p)`,
/// else 255 (background).
///
/// Integral images make the window sums O(1) per pixel regardless of
/// `w`, so total work is O(W*H) — same cost as the Lanczos upscale.
fn sauvola_threshold(
    img: &image::ImageBuffer<image::Luma<u8>, Vec<u8>>,
    w_half: usize,
    k: f32,
) -> image::ImageBuffer<image::Luma<u8>, Vec<u8>> {
    let width = img.width() as usize;
    let height = img.height() as usize;
    let r: f32 = 128.0;

    // Build (W+1) × (H+1) integral images for sum and sum-of-squares.
    // u64 so we never overflow on 4000×4000 inputs.
    let stride = width + 1;
    let mut s_sum = vec![0u64; stride * (height + 1)];
    let mut s_sq = vec![0u64; stride * (height + 1)];
    for y in 0..height {
        let mut row_sum: u64 = 0;
        let mut row_sq: u64 = 0;
        for x in 0..width {
            let p = img.get_pixel(x as u32, y as u32).0[0] as u64;
            row_sum += p;
            row_sq += p * p;
            let idx_below = (y + 1) * stride + (x + 1);
            let idx_above = y * stride + (x + 1);
            s_sum[idx_below] = s_sum[idx_above] + row_sum;
            s_sq[idx_below] = s_sq[idx_above] + row_sq;
        }
    }

    // Window sums in O(1) via the four corner trick.
    let sum_in = |x0: usize, y0: usize, x1: usize, y1: usize, table: &[u64]| -> u64 {
        // x0,y0 inclusive bottom-left; x1,y1 EXCLUSIVE top-right.
        let a = table[y0 * stride + x0];
        let b = table[y0 * stride + x1];
        let c = table[y1 * stride + x0];
        let d = table[y1 * stride + x1];
        d + a - b - c
    };

    let mut out = image::ImageBuffer::new(width as u32, height as u32);
    for y in 0..height {
        let y0 = y.saturating_sub(w_half);
        let y1 = (y + w_half + 1).min(height);
        for x in 0..width {
            let x0 = x.saturating_sub(w_half);
            let x1 = (x + w_half + 1).min(width);
            let n = ((x1 - x0) * (y1 - y0)) as f32;
            let s = sum_in(x0, y0, x1, y1, &s_sum) as f32;
            let s2 = sum_in(x0, y0, x1, y1, &s_sq) as f32;
            let mean = s / n;
            // variance = E[x^2] - E[x]^2; clamp to 0 to dodge fp jitter.
            let var = (s2 / n - mean * mean).max(0.0);
            let std = var.sqrt();
            let t = mean * (1.0 + k * (std / r - 1.0));
            let p = img.get_pixel(x as u32, y as u32).0[0] as f32;
            // p ≤ T → foreground (text). Tesseract reads black-on-white.
            // The `≤` (not `<`) handles the degenerate uniform-black
            // window case: when mean=0 → T=0; a strictly-less compare
            // would classify a pure-black region as background.
            let v = if p <= t { 0u8 } else { 255u8 };
            out.put_pixel(x as u32, y as u32, image::Luma([v]));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Struct sanity — confirms field names match what the parse pipe
    /// expects to read.
    #[test]
    fn raw_text_struct_shape() {
        let r = RawText {
            text: "TOTAL $12.34".into(),
            confidence: 0.85,
            engine: "tesseract".into(),
        };
        assert_eq!(r.text, "TOTAL $12.34");
        assert!((r.confidence - 0.85).abs() < 1e-6);
        assert_eq!(r.engine, "tesseract");
    }

    /// Sidecar discovery — returns None when no env override is set and
    /// the workspace path doesn't exist. Test passes either way (the
    /// workspace path may or may not be populated depending on whether
    /// `tools/tv-ocr-vision/build.sh` has been run); we just exercise
    /// the lookup chain doesn't panic.
    #[test]
    fn find_vision_binary_no_panic() {
        let _ = find_vision_binary();
    }

    /// TV_OCR_VISION_BIN points to a real file → that path is returned.
    /// Use the test binary itself as the "real file" so we know it
    /// exists. Restore the env var afterward.
    ///
    /// Combined env-var test — `env::set_var` is process-global, so
    /// splitting the override-honors-real-path and override-rejects-bad-path
    /// cases into separate `#[test]` fns lets cargo's parallel runner
    /// race them and flake. One serialized test covers both code paths.
    ///
    /// SAFETY: We acquire a process-local mutex to fence against ANY other
    /// test in this crate that touches `TV_OCR_VISION_BIN`. Restoring the
    /// prior value before asserting prevents a panic from poisoning the
    /// env for subsequent tests.
    #[test]
    fn find_vision_binary_env_var_override() {
        use std::sync::Mutex;
        // Process-local lock. `static` Mutex is initialized lazily; no
        // global state escapes the test binary.
        static ENV_LOCK: Mutex<()> = Mutex::new(());
        // PoisonError just means a previous test panicked while holding
        // the lock — recover and proceed (the env was already restored
        // by THAT test's own cleanup, so we're not poisoning state).
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());

        let prior = std::env::var("TV_OCR_VISION_BIN").ok();
        let restore = |prior: &Option<String>| match prior {
            Some(v) => unsafe {
                std::env::set_var("TV_OCR_VISION_BIN", v);
            },
            None => unsafe {
                std::env::remove_var("TV_OCR_VISION_BIN");
            },
        };

        // ── Case 1: env points at a REAL file → return it verbatim ──
        let real_file = std::env::current_exe().expect("test binary path");
        unsafe {
            std::env::set_var("TV_OCR_VISION_BIN", &real_file);
        }
        let got_real = find_vision_binary();

        // ── Case 2: env points at a non-existent file → fall through ──
        let bogus = "/no/such/path/tv-ocr-vision-does-not-exist";
        unsafe {
            std::env::set_var("TV_OCR_VISION_BIN", bogus);
        }
        let got_bogus = find_vision_binary();

        // Restore BEFORE asserting so a failure here doesn't leave the
        // env in a polluted state for later tests.
        restore(&prior);

        assert_eq!(
            got_real,
            Some(real_file),
            "explicit env override pointing at a real file must win"
        );
        assert_ne!(
            got_bogus
                .as_deref()
                .map(|p| p.to_string_lossy().into_owned()),
            Some(bogus.to_string()),
            "bogus env-var path must not be returned"
        );
    }

    /// Sauvola sanity — a half-black/half-white test image binarizes
    /// to mostly the same shape (pure-black region stays foreground,
    /// pure-white stays background). Verifies the integral-image math
    /// + thresholding doesn't invert or scramble the image.
    #[test]
    fn sauvola_preserves_high_contrast_bands() {
        let mut img = image::ImageBuffer::<image::Luma<u8>, _>::new(40, 40);
        for y in 0..40 {
            for x in 0..40 {
                let v = if x < 20 { 0u8 } else { 255u8 };
                img.put_pixel(x, y, image::Luma([v]));
            }
        }
        let out = sauvola_threshold(&img, 5, 0.34);
        // Center of the left band should remain "foreground" (0).
        assert_eq!(out.get_pixel(10, 20).0[0], 0);
        // Center of the right band should remain "background" (255).
        assert_eq!(out.get_pixel(30, 20).0[0], 255);
    }

    /// Sauvola degenerate case — uniform image must produce a uniform
    /// output (no spurious noise from numerical instability when σ=0).
    #[test]
    fn sauvola_uniform_image_stays_uniform() {
        let img = image::ImageBuffer::<image::Luma<u8>, _>::from_pixel(20, 20, image::Luma([200]));
        let out = sauvola_threshold(&img, 4, 0.34);
        // With p=200, mean=200, std=0 → T = 200 * (1 + 0.34*(0/128 - 1))
        //   = 200 * (1 - 0.34) = 132. p=200 > 132 → background (255).
        // Either way, all pixels must have the same value.
        let first = out.get_pixel(0, 0).0[0];
        for y in 0..20 {
            for x in 0..20 {
                assert_eq!(out.get_pixel(x, y).0[0], first);
            }
        }
    }

    /// Sauvola degenerate: 1×1 image. Window math should not div-by-zero
    /// even though every dim < window. Output is one pixel either 0 or
    /// 255 — just don't panic.
    #[test]
    fn sauvola_handles_1x1_image() {
        let img = image::ImageBuffer::<image::Luma<u8>, _>::from_pixel(1, 1, image::Luma([128]));
        let out = sauvola_threshold(&img, 5, 0.34);
        assert_eq!(out.width(), 1);
        assert_eq!(out.height(), 1);
    }

    /// Sauvola degenerate: very narrow 1×N strip (a column scan).
    /// Tests that window-edge clamping (`saturating_sub`, `min`) works
    /// when one dim is smaller than the half-window.
    ///
    /// Strip: alternating dark + light "text-like" pattern. Sauvola
    /// only marks pixels darker than the LOCAL mean as foreground —
    /// uniform regions stay background regardless of luminance, which
    /// is by design (no contrast → no text).
    #[test]
    fn sauvola_handles_narrow_strip() {
        let img = image::ImageBuffer::<image::Luma<u8>, _>::from_fn(1, 20, |_, y| {
            // Mostly light (200) with two dark "ink" rows at y=5 and y=15.
            image::Luma([if y == 5 || y == 15 { 30 } else { 200 }])
        });
        let out = sauvola_threshold(&img, 5, 0.34);
        // Output is the same shape.
        assert_eq!(out.width(), 1);
        assert_eq!(out.height(), 20);
        // The dark "ink" rows should be foreground (0) — they're well
        // below the local mean of the mostly-light window.
        assert_eq!(
            out.get_pixel(0, 5).0[0],
            0,
            "dark pixel surrounded by light must be foreground"
        );
        assert_eq!(
            out.get_pixel(0, 15).0[0],
            0,
            "dark pixel surrounded by light must be foreground"
        );
        // A typical light row is background (255).
        assert_eq!(out.get_pixel(0, 0).0[0], 255);
    }

    /// Sauvola w_half=0: window is just the pixel itself. With k=0.34
    /// and R=128, T = p * (1 + 0.34 * (0/128 - 1)) = p * 0.66.
    /// Then p ≤ 0.66 * p ⇒ p ≤ 0 — only the zero pixel is foreground.
    #[test]
    fn sauvola_zero_window_degenerates_predictably() {
        let mut img = image::ImageBuffer::<image::Luma<u8>, _>::new(3, 1);
        img.put_pixel(0, 0, image::Luma([0]));
        img.put_pixel(1, 0, image::Luma([128]));
        img.put_pixel(2, 0, image::Luma([255]));
        let out = sauvola_threshold(&img, 0, 0.34);
        assert_eq!(out.get_pixel(0, 0).0[0], 0, "0 pixel: 0 ≤ 0 → foreground");
        assert_eq!(
            out.get_pixel(1, 0).0[0],
            255,
            "128 pixel: 128 > 84.48 → background"
        );
        assert_eq!(
            out.get_pixel(2, 0).0[0],
            255,
            "255 pixel: 255 > 168 → background"
        );
    }

    /// End-to-end smoke — `run()` returns Ok or one of the documented
    /// error variants on a 1×1 PNG. CI without tesseract installed hits
    /// ModelsMissing; CI with tesseract installed reaches the engine.
    /// Both branches are acceptable — this pins the error-mapping
    /// contract, not the OCR output.
    #[test]
    fn run_returns_models_missing_when_tesseract_absent_or_succeeds() {
        let png: &[u8] = &[
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00,
            0x00, 0x1f, 0x15, 0xc4, 0x89, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x44, 0x41, 0x54, 0x78,
            0x9c, 0x62, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00,
            0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
        ];
        let r = run(png, Path::new("/nonexistent"));
        match r {
            Ok(_) => {}
            Err(OcrError::ModelsMissing { .. }) => {}
            Err(OcrError::Engine(_)) => {}
            Err(other) => panic!("unexpected error variant: {other:?}"),
        }
    }
}
