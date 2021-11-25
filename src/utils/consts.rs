// https://github.com/rust-lang/rust/blob/master/library/std/src/sys_common/io.rs#L1
pub const DEFAULT_BUF_SIZE: usize = if cfg!(target_os = "espidf") {
    512
} else {
    8 * 1024
};

// Since Linux 2.6.11, the pipe capacity is 16 pages
pub const DEFAULT_PIPE_CAP: usize = 16 * 4096;
