#![cfg_attr(doc, feature(doc_cfg))]

//! Realm's high performance IO collections.
//!
//! ## Example
//!
//! ```no_run
//! async {
//!     use tokio::net::TcpStream;
//!     use realm_io::{bidi_copy, bidi_zero_copy, bidi_copy_buf};
//!     use realm_io::{Pipe, CopyBuffer};
//!
//!     let mut left = TcpStream::connect("abc").await.unwrap();
//!     let mut right = TcpStream::connect("def").await.unwrap();
//!
//!     // direct copy     
//!     bidi_copy(&mut left, &mut right).await;
//!
//!     // zero copy
//!     bidi_zero_copy(&mut left, &mut right).await;
//!
//!     // use custom buffer(AsMut<[u8]>)
//!     let buf1 = CopyBuffer::new(vec![0; 0x2000]);
//!     let buf2 = CopyBuffer::new(vec![0; 0x2000]);
//!     bidi_copy_buf(&mut left, &mut right, buf1, buf2).await;
//!
//!     // use custom buffer(Pipe or &mut Pipe)
//!     let buf1 = CopyBuffer::new(Pipe::new().unwrap());
//!     let buf2 = CopyBuffer::new(Pipe::new().unwrap());
//!     bidi_copy_buf(&mut left, &mut right, buf1, buf2).await;
//! };
//! ```
//!
//! ## About Brutal Shutdown
//!
//! By default, [`bidi_copy_buf`] and other IO functions perform a **graceful shutdown**.
//!
//! With the feature `brutal-shutdown` enabled, these IO functions will decide to
//! perform a **brutal shutdown** once a `FIN` packet reaches, which will forcefully
//! close two connections on both sides without waiting for a reply packet.
//!
//! This is helpful when handling connections from a poorly implemented client or server,
//! which may never shutdown its write side nor close the underlying socket.
//!

mod buf;
mod mem_copy;
mod bidi_copy;

pub use buf::{AsyncIOBuf, CopyBuffer};
pub use bidi_copy::bidi_copy_buf;
pub use mem_copy::{bidi_copy, buf_size, set_buf_size};

#[cfg(target_os = "linux")]
mod linux;

#[cfg(any(target_os = "linux", doc))]
#[cfg_attr(doc, doc(cfg(target_os = "linux")))]
pub use linux::{
    AsyncRawIO, mmsg,
    zero_copy::{Pipe, bidi_zero_copy, pipe_size, set_pipe_size},
};

#[cfg(any(feature = "peek", doc))]
#[cfg_attr(doc, doc(cfg(feature = "peek")))]
pub mod peek;

#[cfg(any(feature = "statistic", doc))]
#[cfg_attr(doc, doc(cfg(feature = "statistic")))]
pub mod statistic;
