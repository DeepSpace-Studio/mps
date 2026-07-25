use std::cell::{Cell, RefCell};
use std::ffi::CString;
use std::os::raw::c_char;

thread_local! {
    static LAST_ERROR_CODE: Cell<u32> = const { Cell::new(0) };
    static LAST_ERROR_MESSAGE: RefCell<CString> = RefCell::new(CString::new("ok").expect("static string has no nul"));
}

pub const ERR_OK: u32 = 0;
pub const ERR_NULL_POINTER: u32 = 1;
pub const ERR_INVALID_ARGUMENT: u32 = 2;
pub const ERR_NOT_FOUND: u32 = 3;
pub const ERR_CAPACITY: u32 = 4;
pub const ERR_UNSUPPORTED: u32 = 5;
pub const ERR_INTERNAL: u32 = 6;

pub fn clear_error() {
    set_error(ERR_OK, "ok");
}

pub fn set_error(code: u32, message: &str) {
    LAST_ERROR_CODE.with(|cell| cell.set(code));
    LAST_ERROR_MESSAGE.with(|cell| {
        let sanitized = message.replace('\0', " ");
        if let Ok(value) = CString::new(sanitized) {
            *cell.borrow_mut() = value;
        }
    });
}

/// Current thread's last error code (`ERR_OK` when no error).
pub fn error_code() -> u32 {
    LAST_ERROR_CODE.with(Cell::get)
}

/// Current thread's last error message as a NUL-terminated C string.
///
/// The returned pointer is owned by the thread-local slot and is invalidated
/// by the next `set_error`/`clear_error` call on the same thread.
pub fn error_message() -> *const c_char {
    LAST_ERROR_MESSAGE.with(|cell| cell.borrow().as_ptr())
}
