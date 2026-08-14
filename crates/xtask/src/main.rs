//! Workspace automation helper (OPTIMIZATION.md §N3).
//!
//! Subcommands:
//!   `dump-metrics`  →  regenerates `crates/mps-web/src/metrics.rs`
//!   `gen-java`       →  scans `#[java_struct]`/`#[java_enum]` annotations and
//!                        generates Java value classes under `test21/.../ffi/`

mod gen_java;

use std::path::Path;
use std::process::ExitCode;

use walkdir::WalkDir;

pub(crate) const JAVA_PACKAGE_DEFAULT: &str = "org.polaris2023.mps.ffi";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(sub) = args.next() else {
        eprintln!(
            "xtask: missing subcommand. Expected one of: dump-metrics, gen-java\n\
             Usage: cargo run -p xtask -- dump-metrics\n\
             Usage: cargo run -p xtask -- gen-java [output_dir]"
        );
        return ExitCode::from(2);
    };

    let workspace_root = find_workspace_root().expect("workspace root");

    match sub.as_str() {
        "dump-metrics" => match dump_metrics(&workspace_root) {
            Ok(report) => {
                println!("{report}");
                ExitCode::from(0)
            }
            Err(err) => {
                eprintln!("xtask dump-metrics failed: {err}");
                ExitCode::from(1)
            }
        },
        "gen-java" => {
            let output_dir = args.next();
            match gen_java::run(&workspace_root, output_dir.as_deref()) {
                Ok(report) => {
                    println!("{report}");
                    ExitCode::from(0)
                }
                Err(err) => {
                    eprintln!("xtask gen-java failed: {err}");
                    ExitCode::from(1)
                }
            }
        }
        other => {
            eprintln!(
                "xtask: unknown subcommand {other:?}. Expected one of: dump-metrics, gen-java."
            );
            ExitCode::from(2)
        }
    }
}

/// Locate the workspace root by walking up from CARGO_MANIFEST_DIR (which
/// points at `crates/xtask/` when running via `cargo run -p xtask`) until a
/// `Cargo.toml` containing `[workspace]` is found.
fn find_workspace_root() -> Option<std::path::PathBuf> {
    use std::path::PathBuf;
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR")?);
    let mut dir = Some(manifest.as_path());
    while let Some(p) = dir {
        let toml = p.join("Cargo.toml");
        if toml.is_file() {
            let content = std::fs::read_to_string(&toml).ok()?;
            if content.contains("[workspace]") {
                return Some(p.to_path_buf());
            }
        }
        dir = p.parent();
    }
    None
}

/// Count `#[test]` lines across the test crate's `src/` tree.
fn count_tests(workspace_root: &Path) -> usize {
    let test_dir = workspace_root.join("crates/mps-test/src");
    let mut count = 0;
    for entry in WalkDir::new(&test_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        // Matches `#[test]` (possibly with arbitrary whitespace inside the
        // brackets). Trims to entire line for cheap check.
        count += content
            .lines()
            .filter(|line| line.trim() == "#[test]")
            .count();
    }
    count
}

/// Count `jni!(` and `jni_e_c!(` token-starts in `crates/mps-jni/src/lib.rs`.
fn count_jni_methods(workspace_root: &Path) -> usize {
    let lib = workspace_root.join("crates/mps-jni/src/lib.rs");
    let Ok(content) = std::fs::read_to_string(&lib) else {
        return 0;
    };
    let mut count = 0;
    for line in content.lines() {
        // simple contains-check. We accept false positives only if a
        // comment mentions `jni!(:` literally — unlikely.
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            continue;
        }
        if trimmed.contains("jni!(") || trimmed.contains("jni_e_c!(") {
            count += 1;
        }
    }
    count
}

/// Count `pub extern "C"` declarations across `crates/mps-core/src/rapier/**/*.rs`.
fn count_core_ffi(workspace_root: &Path) -> usize {
    let dir = workspace_root.join("crates/mps-core/src/rapier");
    let mut count = 0;
    for entry in WalkDir::new(&dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        // Match `pub extern "C" fn ...` (allowing for `unsafe` between pub
        // and extern, and either "C" or C without quotes is invalid so ignore).
        // Simple line-start anchor.
        for line in content.lines() {
            let t = line.trim_start();
            // Accept: pub extern "C" fn, pub(crate) extern "C" fn,
            // pub unsafe extern "C" fn, pub(crate) unsafe extern "C" fn.
            if (t.starts_with("pub extern \"C\"")
                || t.starts_with("pub(crate) extern \"C\"")
                || t.starts_with("pub unsafe extern \"C\"")
                || t.starts_with("pub(crate) unsafe extern \"C\""))
                && t.contains("fn ")
            {
                count += 1;
            }
        }
    }
    count
}

/// Generate `crates/mps-web/src/metrics.rs` from the latest counts.
fn dump_metrics(workspace_root: &Path) -> Result<String, String> {
    let tests = count_tests(workspace_root);
    let jni_methods = count_jni_methods(workspace_root);
    let core_ffi = count_core_ffi(workspace_root);

    let body = format!(
        "// Auto-generated by `cargo run -p xtask -- dump-metrics`.\n\
         // Do NOT edit by hand — re-run the command above after large changes\n\
         // (OPTIMIZATION.md §N3).\n\
         //\n\
         // Source of truth:\n\
         //   TEST_COUNT       = `grep -rh '#[test]' crates/mps-test/src/ | wc -l`\n\
         //   JNI_METHOD_COUNT= `grep -cE 'jni!\\\\(|jni_e_c!\\\\(' crates/mps-jni/src/lib.rs`\n\
         //   CORE_FFI_COUNT  = `grep -rhE '^pub extern \"C\"' crates/mps-core/src/rapier/ | wc -l`\n\
         \n\
         /// Total number of `#[test]` items in `mps-test`, as a `&'static str` so it\n\
         /// can be inserted into `view!` literals directly (`{{ (TEST_COUNT) }}`).\n\
         pub const TEST_COUNT: &str = \"{tests}\";\n\
         /// Total number of `jni!(`/`jni_e_c!(` method entries in `mps-jni`.\n\
         pub const JNI_METHOD_COUNT: &str = \"{jni_methods}\";\n\
         /// Total number of `pub extern \"C\" fn` declarations in `mps-core/rapier`.\n\
         pub const CORE_FFI_COUNT: &str = \"{core_ffi}\";\n"
    );

    let out_path = workspace_root.join("crates/mps-web/src/metrics.rs");
    std::fs::write(&out_path, &body).map_err(|e| format!("write {}: {e}", out_path.display()))?;

    Ok(format!(
        "xtask: wrote {out}\n  TEST_COUNT       = {tests}\n  JNI_METHOD_COUNT = {jni_methods}\n  CORE_FFI_COUNT   = {core_ffi}",
        out = out_path.display()
    ))
}
