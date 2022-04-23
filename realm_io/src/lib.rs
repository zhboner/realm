#![feature(ready_macro)]

mod buf;
mod mem_copy;
mod bidi_copy;

#[cfg(target_os = "linux")]
mod zero_copy;
#[cfg(target_os = "linux")]
pub use zero_copy::{Pipe, AsyncRawIO};

pub use buf::{AsyncIOBuf, CopyBuffer};
pub use bidi_copy::bidi_copy_buf;
