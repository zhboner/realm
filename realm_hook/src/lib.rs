//! Realm's flexible hooks.
//!
//! ## Pre-connect Hook
//!
//! [`first_pkt_len`](pre_conn::first_pkt_len)
//!
//! [`decide_remote_idx`](pre_conn::decide_remote_idx)
//!

pub mod pre_conn;

macro_rules! call_ffi {
    ($dylib: expr, $symbol: expr => $t: ty $(, $arg: expr)*) => {
        unsafe {
            let fp = $dylib.get().unwrap().get::<$t>($symbol).unwrap();
            fp($($arg,)*)
        }
    };
}

pub(crate) use call_ffi;
