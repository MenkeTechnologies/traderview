// tv-ocr-vision: read an image from stdin, run Apple's Vision framework
// VNRecognizeTextRequest, emit one JSON object on stdout.
//
// All processing is on-device using the Apple Neural Engine. No network,
// no API charges, no model downloads — Vision ships with macOS.

import Foundation
import Vision
import CoreImage
import ImageIO

let stderrHandle = FileHandle.standardError

func emitError(_ msg: String) -> Never {
    // Try to write a structured error so the Rust side can parse it,
    // but never block on a failing stderr write — fall through to exit.
    let payload: [String: Any] = ["error": msg]
    if let data = try? JSONSerialization.data(withJSONObject: payload),
       let nl = "\n".data(using: .utf8) {
        stderrHandle.write(data)
        stderrHandle.write(nl)
    }
    exit(2)
}

// ---------------------------------------------------------------------------
// 1) Read the entire image payload from stdin. The Rust caller pipes raw
//    image bytes (JPEG, PNG, HEIC, TIFF — anything CIImage can decode).
// ---------------------------------------------------------------------------
let inputData = FileHandle.standardInput.readDataToEndOfFile()
if inputData.isEmpty {
    emitError("empty stdin — caller piped no bytes")
}

guard let image = CIImage(data: inputData) else {
    emitError("CIImage rejected the input bytes — unsupported format or corrupt image")
}

// ---------------------------------------------------------------------------
// 2) Configure the Vision request for receipt-grade OCR.
//
//    .accurate          — heavier transformer-based model; significantly
//                         better than .fast for noisy phone photos of
//                         crumpled / faded receipts.
//    usesLanguageCorrection — re-ranks ambiguous tokens against an English
//                         language model (fixes "0" vs "O", "1" vs "l", etc.)
//    recognitionLanguages — explicit en-US so Vision doesn't waste cycles
//                         on multi-language guess.
//    minimumTextHeight  — leave at default (0.0). Receipts have tiny
//                         line-item rows; clamping would drop them.
// ---------------------------------------------------------------------------
let request = VNRecognizeTextRequest()
request.recognitionLevel = .accurate
request.usesLanguageCorrection = true
request.recognitionLanguages = ["en-US"]

let handler = VNImageRequestHandler(ciImage: image, options: [:])
do {
    try handler.perform([request])
} catch {
    emitError("Vision handler failed: \(error.localizedDescription)")
}

// ---------------------------------------------------------------------------
// 3) Walk the observations, take the top candidate per line, emit JSON.
//
//    bbox: Vision's coordinates are normalized [0,1] with origin at the
//    BOTTOM-LEFT — exposed verbatim, the Rust side flips Y if it wants
//    top-left convention.
// ---------------------------------------------------------------------------
let observations = request.results ?? []

var lines: [[String: Any]] = []
for obs in observations {
    guard let cand = obs.topCandidates(1).first else { continue }
    let bb = obs.boundingBox
    lines.append([
        "text":       cand.string,
        "confidence": cand.confidence,
        "bbox":       [bb.origin.x, bb.origin.y, bb.size.width, bb.size.height],
    ])
}

// Aggregate confidence — mean of per-line top-candidate scores. The Rust
// side uses this to decide whether to re-OCR with a different engine
// or flag the receipt for manual review.
var confidenceSum: Double = 0
var confidenceCount = 0
var confidenceMin: Double = 1.0
for line in lines {
    if let c = line["confidence"] as? Float {
        let cd = Double(c)
        confidenceSum += cd
        confidenceCount += 1
        if cd < confidenceMin { confidenceMin = cd }
    }
}
let confidenceMean = confidenceCount > 0 ? confidenceSum / Double(confidenceCount) : 0.0
if confidenceCount == 0 { confidenceMin = 0.0 }

let payload: [String: Any] = [
    "engine":          "apple_vision",
    "lines":           lines,
    "line_count":      lines.count,
    "confidence_mean": confidenceMean,
    "confidence_min":  confidenceMin,
]

guard let out = try? JSONSerialization.data(withJSONObject: payload, options: []) else {
    emitError("failed to serialize result JSON")
}
FileHandle.standardOutput.write(out)
