//! Places the vendored herdr beside the zeddy binary.
//!
//! zeddy resolves its backend by path — `<dir of the zeddy executable>/herdr` —
//! and never through `PATH`, so the executable has to actually be there. This
//! copies it, and fails the build if it is not vendored, because a zeddy that
//! builds and then cannot start a session is worse than one that does not build.
//!
//! This does not reach the network. Fetching is `vendor/herdr/fetch.sh`, run by
//! hand when the pin moves.

use std::path::{Path, PathBuf};

fn main() {
    let target = std::env::var("TARGET").expect("TARGET");
    let root = workspace_root();
    let vendored = root.join("vendor/herdr").join(&target).join("herdr");

    println!("cargo::rerun-if-changed={}", vendored.display());

    if !vendored.is_file() {
        println!(
            "cargo::error=no herdr for {target} at {}. Run `sh vendor/herdr/fetch.sh`.",
            vendored.display()
        );
        return;
    }

    let beside = out_dir_binary_dir().join("herdr");
    if let Err(err) = std::fs::copy(&vendored, &beside) {
        println!("cargo::error=cannot place herdr at {}: {err}", beside.display());
    }
}

fn workspace_root() -> PathBuf {
    // `CARGO_MANIFEST_DIR` is `crates/zeddy`; the workspace is two above it.
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    manifest.parent().and_then(Path::parent).expect("the workspace root").to_owned()
}

/// Cargo gives a build script `OUT_DIR`, not the directory the binary lands in.
/// The binary directory is three levels up: `…/<profile>/build/<pkg>-<hash>/out`.
fn out_dir_binary_dir() -> PathBuf {
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    out.ancestors().nth(3).expect("the profile directory").to_owned()
}
