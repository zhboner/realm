# Realm Hook

[![crates.io](https://img.shields.io/crates/v/realm_hook.svg)](https://crates.io/crates/realm_hook)
[![Released API docs](https://docs.rs/realm_hook/badge.svg)](https://docs.rs/realm_hook)

Realm's flexible hooks.

## Pre-connect Hook

```c
// Get the required length of first packet.
uint32_t realm_first_pkt_len();
```

```c
// Get the index of the selected remote peer.
//
// Remote peers are defined in `remote`(default) and `extra_remotes`(extended),
// where there should be at least 1 remote peer whose idx is 0.
//
// idx < 0 means **ban**.
// idx = 0 means **default**.
int32_t realm_decide_remote_idx(int32_t max_remote_idx, const char *pkt);
```
