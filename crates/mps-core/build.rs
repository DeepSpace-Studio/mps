use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=cbindgen.toml");

    let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    // Header generation is delegated to `mps-build-common` (OPTIMIZATION.md §7).
    mps_build_common::run_cbindgen(&crate_dir, "rigid_body.h");
}
