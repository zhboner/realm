//! Realm's flexible hooks.

use once_cell::unsync::OnceCell;
use libloading::Library;

static mut DYLIB: OnceCell<Library> = OnceCell::new();

/// Load a dynamic library.
///
/// This is not thread-safe and must be called before interacting with FFI.
pub fn load_dylib(path: &str) {
    unsafe {
        DYLIB.set(Library::new(path).unwrap()).unwrap();
    }
}

macro_rules! call_ffi {
    ($symbol: expr, $t: ty $(, $arg: expr)*) => {
        unsafe {
            let fp = DYLIB.get().unwrap().get::<$t>($symbol).unwrap();
            fp($($arg)*)
        }
    };
}

/// Get the required length of first packet.
pub fn first_pkt_len() -> u32 {
    call_ffi!(b"realm_first_pkt_len", unsafe extern "C" fn() -> u32)
}

/// Get the index of the selected remote peer.
///
/// Remote peers are defined in `remote`(default) and `extra_remotes`(extended),
/// where there should be at least 1 remote peer whose idx is 0.
///
/// idx < 0 means **ban**.
/// idx = 0 means **default**.
pub fn decide_remote(buf: *const u8) -> i32 {
    call_ffi!(b"realm_decide_remote", unsafe extern "C" fn(*const u8) -> i32, buf)
}
