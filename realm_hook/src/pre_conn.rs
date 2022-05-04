//! Pre-connect hook.

use once_cell::unsync::OnceCell;
use libloading::Library;

use super::call_ffi;

static mut LOAD: bool = false;
static mut DYLIB: OnceCell<Library> = OnceCell::new();

/// Load a dynamic library.
///
/// This is not thread-safe and must be called before interacting with FFI.
pub fn load_dylib(path: &str) {
    unsafe {
        DYLIB.set(Library::new(path).unwrap()).unwrap();
        LOAD = true;
    }
}

/// Check if the dynamic library is loaded.
pub fn is_loaded() -> bool {
    unsafe { LOAD }
}

/// Get the required length of first packet.
pub fn first_pkt_len() -> u32 {
    call_ffi!(DYLIB, b"realm_first_pkt_len" => unsafe extern "C" fn() -> u32)
}

/// Get the index of the selected remote peer.
///
/// Remote peers are defined in `remote`(default) and `extra_remotes`(extended),
/// where there should be at least 1 remote peer whose idx is 0.
///
/// idx < 0 means **ban**.
/// idx = 0 means **default**.
pub fn decide_remote_idx(idx: i32, buf: *const u8) -> i32 {
    call_ffi!(
        DYLIB, b"realm_decide_remote_idx" => unsafe extern "C" fn(i32, *const u8) -> i32,
        idx, buf
    )
}
