// Auto-split from original `ffi/types.rs` (2464 lines) — see OPTIMIZATION.md §2.
// This mod.rs re-exports every legacy submodule so `pub use types::*;` (in
// `ffi::mod`) and all downstream `mps_formula::ffi::types::Vec3` paths
// continue to resolve byte-for-byte. The split is along section-header
// boundaries that already existed in the original file.

pub(crate) mod chaos;
pub(crate) mod core;
pub(crate) mod events;
pub(crate) mod optics;
pub(crate) mod physics;
pub(crate) mod plasma;
pub(crate) mod superfluid;

pub use chaos::*;
pub use core::*;
pub use events::*;
pub use optics::*;
pub use physics::*;
pub use plasma::*;
pub use superfluid::*;
