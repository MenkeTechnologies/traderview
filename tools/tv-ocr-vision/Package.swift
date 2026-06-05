// swift-tools-version:5.9
//
// tv-ocr-vision — Vision-framework OCR sidecar for traderview.
//
// Stdin:  arbitrary image bytes (JPEG, PNG, HEIC, TIFF — anything CIImage
//         can decode).
// Stdout: JSON {engine, lines:[{text, confidence, bbox:[x,y,w,h]}]}.
//         bbox coordinates are normalized [0,1] with origin at bottom-left
//         (Vision's native convention — the Rust side scales if needed).
// Exit:   0 on success, 2 on decode/Vision failure (with {"error":...}
//         on stderr).
//
// Build:   swift build -c release  (or: ./build.sh)
// Output:  .build/release/tv-ocr-vision

import PackageDescription

let package = Package(
    name: "tv-ocr-vision",
    platforms: [.macOS(.v13)],
    targets: [
        .executableTarget(
            name: "tv-ocr-vision",
            path: "Sources/tv-ocr-vision"
        )
    ]
)
