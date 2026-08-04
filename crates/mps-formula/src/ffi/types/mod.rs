// Auto-split from original `ffi/types.rs` (2464 lines) — see OPTIMIZATION.md §2.
// This mod.rs re-exports every legacy submodule so `pub use types::*;` (in
// `ffi::mod`) and all downstream `mps_formula::ffi::types::Vec3` paths
// continue to resolve byte-for-byte. The split is along section-header
// boundaries that already existed in the original file.

pub(crate) mod core;
pub(crate) mod physics;
pub(crate) mod chaos;
pub(crate) mod superfluid;
pub(crate) mod optics;
pub(crate) mod plasma;
pub(crate) mod events;

pub use core::*;
pub use physics::*;
pub use chaos::*;
pub use superfluid::*;
pub use optics::*;
pub use plasma::*;
pub use events::*;
