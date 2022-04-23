#![feature(ready_macro)]

mod buf;
mod mem_copy;
mod bidi_copy;

#[cfg(target_os = "linux")]
mod zero_copy;

pub use buf::{AsyncIOBuf, CopyBuffer};
pub use bidi_copy::bidi_copy_buf;
pub use mem_copy::bidi_copy;

#[cfg(target_os = "linux")]
pub use zero_copy::{Pipe, AsyncRawIO, bidi_zero_copy};
