use crate::ffi::RcBool;

pub const RC_ABI_VERSION: u32 = 1;

#[unsafe(no_mangle)]
pub extern "C" fn rc_abi_version() -> u32 {
    RC_ABI_VERSION
}

#[unsafe(no_mangle)]
pub extern "C" fn rc_abi_supports_ffm() -> RcBool {
    RcBool::TRUE
}

#[unsafe(no_mangle)]
pub extern "C" fn rc_abi_supports_jni() -> RcBool {
    RcBool::TRUE
}
