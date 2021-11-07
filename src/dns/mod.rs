use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "trust-dns")] {
        mod trust_dns;
        pub use trust_dns::*;
    } else {
        mod sys_dns;
        pub use sys_dns::*;
        pub use resolve as resolve_sync;
    }
}
