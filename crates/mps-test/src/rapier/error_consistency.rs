//! Cross-crate consistency test for the `ERR_*` codes (OPTIMIZATION.md §1).
//!
//! DESIGN.md §3.2 mandates that `ERR_OK`/`ERR_NULL_POINTER`/`ERR_INVALID_ARGUMENT`/
//! `ERR_NOT_FOUND`/`ERR_CAPACITY`/`ERR_UNSUPPORTED`/`ERR_INTERNAL` are *independently*
//! declared in both `mps-formula::error` and `mps_core::rapier::error`. This is a
//! deliberate cbindgen workaround — `pub use` is NOT recognised by cbindgen, so each
//! crate must declare the constants itself for them to appear in `rigid_body.h`.
//!
//! The two declarations are currently guarded by *ad-hoc* compile-time `assert!`
//! calls scattered across the crates, which makes failures ambiguous. This module
//! centralises the equality assertion: should anyone bump a value on one side but
//! not the other, this test fails with a precise diff message naming both sites.
//!
//! Risk: zero (read-only test; existing scattered asserts can stay as belt-and-
//! braces).

#[cfg(test)]
mod tests {
    use mps_core::rapier::error as core_err;
    use mps_formula::error as formula_err;

    /// Asserts the seven `ERR_*` constants share identical numeric values in
    /// `mps-formula::error` and `mps_core::rapier::error`.
    ///
    /// If this test fails, the cbindgen-generated `rigid_body.h` for one crate
    /// will disagree with the other on the codes' integer values — pick the
    /// canonical value and update the other crate, do NOT silence this.
    #[test]
    fn err_codes_are_identical_across_formula_and_core() {
        let cases: [(&str, u32, u32); 7] = [
            ("ERR_OK", formula_err::ERR_OK, core_err::ERR_OK),
            ("ERR_NULL_POINTER", formula_err::ERR_NULL_POINTER, core_err::ERR_NULL_POINTER),
            ("ERR_INVALID_ARGUMENT", formula_err::ERR_INVALID_ARGUMENT, core_err::ERR_INVALID_ARGUMENT),
            ("ERR_NOT_FOUND", formula_err::ERR_NOT_FOUND, core_err::ERR_NOT_FOUND),
            ("ERR_CAPACITY", formula_err::ERR_CAPACITY, core_err::ERR_CAPACITY),
            ("ERR_UNSUPPORTED", formula_err::ERR_UNSUPPORTED, core_err::ERR_UNSUPPORTED),
            ("ERR_INTERNAL", formula_err::ERR_INTERNAL, core_err::ERR_INTERNAL),
        ];
        for (name, formula_val, core_val) in cases {
            assert_eq!(
                formula_val, core_val,
                "{name} mismatch: mps-formula::error declares {formula_val} but \
                 mps_core::rapier::error declares {core_val}. These two crates \
                 must keep their ERR_* codes in lockstep (DESIGN.md §3.2 + \
                 OPTIMIZATION.md §1) — pick the canonical value and update both \
                 `pub const` declarations, do NOT silence this test."
            );
        }
    }
}
