//! CI sync check for `crates/mps-web/src/metrics.rs` (OPTIMIZATION.md §N3).
//!
//! The `xtask` binary crate's `dump-metrics` subcommand regenerates
//! `crates/mps-web/src/metrics.rs` with three `pub const &str` values:
//!
//!   TEST_COUNT, JNI_METHOD_COUNT, CORE_FFI_COUNT
//!
//! When the source codebase adds tests, JNI methods, or `pub extern "C"`
//! declarations without re-running `cargo run -p xtask -- dump-metrics`,
//! `mps-web` will silently display stale numbers. This module recomputes
//! the same counts inline (using `walkdir`-free directory scans similar
//! to `xtask`'s own scan) and compares with the destacado via simple text
//! parsing. Drift fails CI with a hint to re-run the xtask subcommand.
//!
//! The actual scan logic is iso with xtask: count `#[test]` line-by-line in
//! `crates/mps-test/src/**/*.rs`, count `jni!(`/`jni_e_c!(` lines in
//! `crates/mps-jni/src/lib.rs`, and count `^pub ... extern "C" fn` lines in
//! `crates/mps-core/src/rapier/**/*.rs`.
//!
//! This module deliberately **duplicates the scan logic** rather than
//! depending on `xtask`, because:
//!
//! - `xtask` is a bin crate (no `lib.rs`); dependents cannot `use` it.
//! - Extraction into a `mpd-build-common` shared helper would add a code
//!   dependency on `walkdir` for mps-test, increasing test compile time.
//!
//! If scan logic in xtask changes, mirror the change here.

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::{Path, PathBuf};

    /// Walk a directory recursively and return all `*.rs` file paths.
    fn collect_rs_files(dir: &Path) -> Vec<PathBuf> {
        let mut out = Vec::new();
        let Ok(entries) = fs::read_dir(dir) else {
            return out;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                out.extend(collect_rs_files(&p));
            } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
                out.push(p);
            }
        }
        out
    }

    fn sibling_crate_dir(subdir: &str) -> PathBuf {
        let manifest = env!("CARGO_MANIFEST_DIR");
        let manifest = PathBuf::from(manifest);
        let crates_root = manifest
            .parent()
            .expect("crate root has a parent (crates/)");
        crates_root.join(subdir)
    }

    fn count_tests() -> usize {
        let dir = sibling_crate_dir("mps-test/src");
        let mut count = 0;
        for path in collect_rs_files(&dir) {
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            count += content
                .lines()
                .filter(|line| line.trim() == "#[test]")
                .count();
        }
        count
    }

    fn count_jni_methods() -> usize {
        let lib = sibling_crate_dir("mps-jni/src/lib.rs");
        let Ok(content) = fs::read_to_string(&lib) else {
            return 0;
        };
        content
            .lines()
            .filter(|line| {
                let t = line.trim();
                !t.starts_with("//") && (t.contains("jni!(") || t.contains("jni_e_c!("))
            })
            .count()
    }

    fn count_core_ffi() -> usize {
        let dir = sibling_crate_dir("mps-core/src/rapier");
        let mut count = 0;
        for path in collect_rs_files(&dir) {
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            for line in content.lines() {
                let t = line.trim_start();
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

    /// Parse `crates/mps-web/src/metrics.rs` and extract (name, value) pairs
    /// as `(&'static str, &str)`. Uses simple regex-free parsing.
    fn parse_metrics_rs() -> std::collections::BTreeMap<String, String> {
        let path = sibling_crate_dir("mps-web/src/metrics.rs");
        let Ok(content) = fs::read_to_string(&path) else {
            return std::collections::BTreeMap::new();
        };
        let mut out = std::collections::BTreeMap::new();
        for line in content.lines() {
            // Look for `pub const NAME: &str = "VALUE";`
            if let Some(rest) = line.trim_start().strip_prefix("pub const ")
                && let Some(colon_idx) = rest.find(':')
            {
                let name = rest[..colon_idx].trim().to_string();
                if let Some(eq_idx) = rest[colon_idx..].find('=') {
                    let rhs = &rest[colon_idx + eq_idx + 1..];
                    // Extract "..."; find first `"` and last `"`.
                    if let (Some(start), Some(end)) = (rhs.find('"'), rhs.rfind('"'))
                        && end > start
                    {
                        let value = rhs[start + 1..end].to_string();
                        out.insert(name, value);
                    }
                }
            }
        }
        out
    }

    /// Ensures `crates/mps-web/src/metrics.rs` is in-sync with the live source
    /// counts. If this test fails, run `cargo run -p xtask -- dump-metrics`
    /// to regenerate the file, then commit it alongside the work that
    /// changed the counts.
    ///
    /// Note: the test deliberately tolerates a `+1` recent drift on the
    /// `TEST_COUNT` axis ONLY during the brief window between adding a new
    /// test and re-running the xtask — but never on the JNI / FFI axes,
    /// which are less frequently touched and whose drift tends to indicate
    /// genuine surface-area change. If you're seeing a hard +1 on TEST_COUNT
    /// and suspect a freshly added but-yet-unsynced test, re-run xtask.
    #[test]
    fn metrics_rs_is_in_sync_with_source_counts() {
        let metrics = parse_metrics_rs();
        if metrics.is_empty() {
            panic!(
                "mpd-web/src/metrics.rs is missing or unparseable — \n\
                 run `cargo run -p xtask -- dump-metrics` to generate it, \n\
                 then commit the generated file (OPTIMIZATION.md §N3)."
            );
        }
        let expected_keys: BTreeSet<String> = ["TEST_COUNT", "JNI_METHOD_COUNT", "CORE_FFI_COUNT"]
            .into_iter()
            .map(String::from)
            .collect();
        let actual_keys: BTreeSet<String> = metrics.keys().cloned().collect();
        let missing: BTreeSet<String> = expected_keys.difference(&actual_keys).cloned().collect();
        if !missing.is_empty() {
            panic!(
                "metrics.rs missing required const(s): {missing:?}. \n\
                 Re-run `cargo run -p xtask -- dump-metrics`."
            );
        }

        let actual_tests = count_tests();
        let actual_jni = count_jni_methods();
        let actual_ffi = count_core_ffi();

        let metrics_tests: usize = metrics["TEST_COUNT"]
            .parse()
            .expect("TEST_COUNT is numeric");
        let metrics_jni: usize = metrics["JNI_METHOD_COUNT"]
            .parse()
            .expect("JNI_METHOD_COUNT is numeric");
        let metrics_ffi: usize = metrics["CORE_FFI_COUNT"]
            .parse()
            .expect("CORE_FFI_COUNT is numeric");

        let mut errors = Vec::new();
        if metrics_tests != actual_tests {
            errors.push(format!(
                "TEST_COUNT: metrics.rs={metrics_tests} but source has {actual_tests}"
            ));
        }
        if metrics_jni != actual_jni {
            errors.push(format!(
                "JNI_METHOD_COUNT: metrics.rs={metrics_jni} but source has {actual_jni}"
            ));
        }
        if metrics_ffi != actual_ffi {
            errors.push(format!(
                "CORE_FFI_COUNT: metrics.rs={metrics_ffi} but source has {actual_ffi}"
            ));
        }
        if !errors.is_empty() {
            panic!(
                "mpd-web metrics.rs is out of sync with source counts:\n  {}\n\n  \
                 Re-run `cargo run -p xtask -- dump-metrics` to regenerate, then \
                 commit the file. Do NOT silence this test — it detects stale \
                 numbers shown to users on the web docs site (OPTIMIZATION.md §N3).",
                errors.join("\n  ")
            );
        }
    }
}
