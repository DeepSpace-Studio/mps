//! Module-mirror CI guard for mps-test ↔ mps-core/mps-cosmos/mps-formula.
//!
//! DESIGN.md mandates that whenever a `mpd-core::rapier::*` submodule is added
//! or renamed, the corresponding `mpd-test/src/rapier/<name>.rs` file must
//! be kept in lockstep; likewise for cosmos. Until now this rule has been a
//! manual convention only — a developer who deleted `core/rapier/foo.rs` but
//! left `mod foo;` dangling got an inscrutable *compile* failure, and a
//! developer who added a new core submodule without a test file got *silent
//! missed coverage*. This module turns that DSL convention into a CI-visible
//! assertion, per OPTIMIZATION.md §8.
//!
//! The rule it enforces, computed by directory listing only (no parsing):
//!
//! ```text
//! let test_rapier  = files(crate "mpd-test/src/rapier/")
//! let core_rapier  = files(crate "mpd-core/src/rapier/")
//! let formula_src  = files(crate "mpd-formula/src/")
//! let expected     = (core_rapier |union| formula_src)
//!                     - {"mod.rs", "lib.rs"}     (structural, no test file)
//! let extra_ok     = {"ffi.rs", "arena_compat.rs"}  // test-only adapters
//! assert!(test_rapier == expected |union| extra_ok)
//! ```
//!
//! For cosmos the rule is simpler (1:1 mirror of `crates/mpd-cosmos/src/`
//! minus `lib.rs`/`ffi.rs`):
//!
//! ```text
//! let test_cosmos = files("mpd-test/src/cosmos/")
//! let cosmos_src  = files("mpd-cosmos/src/")
//! let extra_ok    = {}  // no test-only adapters expected
//! assert!(test_cosmos == cosmos_src - {"lib.rs", "ffi.rs"})
//! ```

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::PathBuf;

    /// List every `*.rs` filename (basename only) inside `dir`.
    fn list_rs_basenames(dir: PathBuf) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        out.insert(name.to_string());
                    }
                }
            }
        }
        out
    }

    /// Resolve a sibling crate directory relative to this crate.
    ///
    /// Uses `CARGO_MANIFEST_DIR` (which points at `crates/mpd-test/` for this
    /// crate) and steps up two levels to reach the workspace root, then down
    /// into the requested sibling crate.
    fn sibling_crate_src(subdir: &str) -> PathBuf {
        let manifest = option_env!("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR is set by cargo during build/test");
        // crates/mpd-test -> crates/<sibling>
        let manifest = PathBuf::from(manifest);
        let crates_root = manifest
            .parent()
            .expect("crate root has a parent (crates/)");
        crates_root.join(subdir)
    }

    /// rapier submodules: `mpd-test::rapier` mirrors the union of
    /// `mpd-core::rapier::*` and `mpd-formula::*` (the latter providing
    /// pure-formula physics domains whose tests live under rapier/ for
    /// historical reasons). The four `STRUCTURAL_OR_EXTRA_ALLOW` files below
    /// are the only legal deviations from the union.
    #[test]
    fn rapier_submodules_mirror_core_and_formula() {
        let test_rapier =
            list_rs_basenames(sibling_crate_src("mps-test/src/rapier"));
        let core_rapier =
            list_rs_basenames(sibling_crate_src("mps-core/src/rapier"));
        let formula_src = list_rs_basenames(sibling_crate_src("mps-formula/src"));

        // Union of what should be mirrored: every name that exists in either
        // core::rapier or formula is expected to have a test file under
        // mpd-test/src/rapier.
        let mut expected: BTreeSet<String> = BTreeSet::new();
        expected.extend(core_rapier);
        expected.extend(formula_src);

        // Structural files that have NO test counterpart by design:
        //   mod.rs    — Rust module declaration in core::rapier
        //   lib.rs    — crate root of formula
        for structural in ["mod.rs", "lib.rs"] {
            expected.remove(structural);
        }

        // Test-side-only files that are NOT mirrored back into core/formula:
        //   ffi.rs          — mps-test has its own minimal FFI helper
        //   arena_compat.rs — ABI lock-in test for shared_arena (OPTIMIZATION §10)
        //   error_consistency.rs  — cross-crate ERR_* consistency test (OPTIMIZATION §1)
        //   verify_module_mirror.rs — THIS FILE (meta CI guard, OPTIMIZATION §8)
        let test_only_extras: BTreeSet<String> = [
            "ffi.rs",
            "arena_compat.rs",
            "error_consistency.rs",
            "verify_module_mirror.rs",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let mut expected_with_extras = expected.clone();
        expected_with_extras.extend(test_only_extras.clone());

        // also: test side may legitimately carry more extras that are NOT in
        // core/formula ONLY if they are explicitly allow-listed above. Any
        // other test-only file is suspicious — likely a stale leftover.
        let orphans_in_test: BTreeSet<String> =
            test_rapier.difference(&expected).cloned().collect();
        let real_orphans: BTreeSet<String> = orphans_in_test
            .difference(&test_only_extras)
            .cloned()
            .collect();

        let missing_in_test: BTreeSet<String> = expected
            .difference(&test_rapier)
            .cloned()
            .collect();

        if !real_orphans.is_empty() || !missing_in_test.is_empty() {
            panic!(
                "rapier mirror drift detected:\n  \
                孤儿 test-files (test has but neither core nor formula defines \
                 such a submodule): {real_orphans:?}\n  \
                缺失 test-files (core/formula has submodule but test has no mirror): \
                 {missing_in_test:?}\n\n  \
                 EDITION.md / DESIGN.md rule: when adding/removing/renaming a \
                 `mpd-core::rapier::*` or `mpd-formula::*` submodule, the \
                 corresponding `mpd-test/src/rapier/<name>.rs` file (and the \
                 `pub mod <name>;` line in mpd-test/src/lib.rs) MUST be kept in \
                 sync. Do not silence this test — fix the drift."
            );
        }
    }

    /// cosmos submodules: `mpd-test::cosmos::*` mirrors `mpd-cosmos/src/*.rs`
    /// modulo the structural `lib.rs` / `ffi.rs`. No test-only extras are
    /// expected today.
    #[test]
    fn cosmos_submodules_mirror_cosmos_src() {
        let test_cosmos =
            list_rs_basenames(sibling_crate_src("mps-test/src/cosmos"));
        let cosmos_src =
            list_rs_basenames(sibling_crate_src("mps-cosmos/src"));

        let mut expected = cosmos_src.clone();
        for structural in ["lib.rs", "ffi.rs"] {
            expected.remove(structural);
        }

        let missing_in_test: BTreeSet<String> = expected
            .difference(&test_cosmos)
            .cloned()
            .collect();
        let orphans_in_test: BTreeSet<String> = test_cosmos
            .difference(&expected)
            .cloned()
            .collect();

        if !missing_in_test.is_empty() || !orphans_in_test.is_empty() {
            panic!(
                "cosmos mirror drift detected:\n  \
                 missing test files (cosmos src has submodule but test has no mirror): \
                 {missing_in_test:?}\n  \
                 孤儿 test files (test has but cosmos src doesn't define): \
                 {orphans_in_test:?}\n"
            );
        }
    }
}
