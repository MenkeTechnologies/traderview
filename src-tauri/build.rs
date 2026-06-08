use std::fs;
use std::path::Path;

fn main() {
    // Emit a tiny JSON blob into the bundled frontend dir so the topbar
    // version chip has a guaranteed-fresh source even when /api/config
    // fails (stale running binary, mid-restart, offline web mode, etc.).
    // CARGO_PKG_VERSION is baked from workspace.package.version, so this
    // file always reflects the workspace version at *build* time —
    // single-source from Cargo.toml.
    let ver = env!("CARGO_PKG_VERSION");
    let out_path = Path::new("../frontend/.version.json");
    let body = format!("{{\"version\":\"{ver}\"}}\n");
    if let Err(e) = fs::write(out_path, body) {
        // Don't fail the build over this — the frontend has REST + npm
        // fallbacks. Just warn.
        println!("cargo:warning=failed to write {}: {e}", out_path.display());
    }
    // Rebuild whenever the version source changes.
    println!("cargo:rerun-if-changed=../Cargo.toml");

    tauri_build::build()
}
