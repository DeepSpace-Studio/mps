// build.rs — copy the `public/` directory (containing index.html) next to the
// binary so Dioxus fullstack's SSR `ServeConfig::build()` can find it at
// runtime via `current_exe().parent().join("public")`.
//
// This makes `cargo build -p mps-web` produce a runnable server without
// requiring the `dx` CLI to bundle assets.
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let src_public = manifest_dir.join("public");

    // OUT_DIR is typically `<target>/<profile>/build/<pkg>-<hash>/out`.
    // Walk up to the profile dir (.. 4 levels) and place `public/` there,
    // next to the binary.
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let profile_dir = out_dir
        .ancestors()
        .nth(3)
        .expect("OUT_DIR should have at least 3 parent dirs to reach the profile dir");

    let dst_public = profile_dir.join("public");

    if !src_public.exists() {
        println!(
            "cargo:warning=mps-web: no `public/` directory found in manifest; SSR will return 404"
        );
        return;
    }

    // Rerun if the public dir contents change.
    println!("cargo:rerun-if-changed=public/");

    copy_dir_recursive(&src_public, &dst_public);
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) {
    std::fs::create_dir_all(dst).ok();
    for entry in std::fs::read_dir(src).expect("failed to read public dir") {
        let entry = entry.expect("bad dir entry");
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir_recursive(&from, &to);
        } else {
            std::fs::copy(&from, &to).ok();
        }
    }
}
