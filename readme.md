[![realm](https://github.com/zhboner/realm/workflows/ci/badge.svg)](https://github.com/zhboner/realm/actions)
[![realm](https://github.com/zhboner/realm/workflows/compile/badge.svg)](https://github.com/zhboner/realm/actions/workflows/cross_compile.yml)
[![realm](https://github.com/zhboner/realm/workflows/release/badge.svg)](https://github.com/zhboner/realm/releases)
[![downloads](https://img.shields.io/github/downloads/zhboner/realm/total?color=green)](https://github.com/zhboner/realm/releases)

[中文说明](https://zhb.me/realm)

<p align="center"><img src="https://raw.githubusercontent.com/zhboner/realm/master/realm.png"/></p>

## Introduction

Realm is a simple, high performance relay server written in rust.

## Features

- Zero configuration. Setup and run in one command.
- Concurrency. Bidirectional concurrent traffic leads to high performance.
- Low resources cost.

## Transports

With `transport` feature, Realm is able to handle [ws/tls/wss] on both sides. This is powered by [libkaminari](https://github.com/zephyrchien/kaminari), which is dramatically faster in comparison to other tools. [Here are some benchmark results](https://github.com/zephyrchien/boring-relay-bench).

## Container

Realm can be run in a container with OCI (like Docker, Podman, Kubernetes, etc), see guides [here](readme.container.md).

## Build Guides

Install rust toolchains with [rustup](https://rustup.rs/).

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Clone this repository

```shell
git clone https://github.com/zhboner/realm
```

Enter the directory and build

```shell
cd realm
git submodule sync && git submodule update --init --recursive
cargo build --release
```

### Build Options

- udp *(enabled by default)*
- trust-dns *(enabled by default)*
- zero-copy *(enabled on linux)*
- transport *(enabled by default)*
- multi-thread *(enabled by default)*
- tfo
- mi-malloc
- jemalloc

See also: `Cargo.toml`

Examples:

```shell
# simple tcp
cargo build --release --no-default-features

# enable other options
cargo build --release --no-default-features --features udp, tfo, zero-copy, trust-dns
```

### Cross Compile

Please refer to [https://rust-lang.github.io/rustup/cross-compilation.html](https://rust-lang.github.io/rustup/cross-compilation.html). You may need to install cross-compilers or other SDKs, and specify them when building the project.

Or have a look at [Cross](https://github.com/cross-rs/cross), it makes things easier.

## Usage

```shell
Realm 2.x [udp][zero-copy][trust-dns][proxy-protocol][transport][multi-thread]
A high efficiency relay tool

USAGE:
    realm [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       show help
    -v, --version    show version
    -d, --daemon     run as a unix daemon
    -u, --udp        force enable udp forward
    -f, --tfo        force enable tcp fast open
    -z, --splice     force enable tcp zero copy

OPTIONS:
    -n, --nofile <limit>                set nofile limit
    -p, --page <number>                 set pipe capacity
    -c, --config <path>                 use config file
    -l, --listen <address>              listen address
    -r, --remote <address>              remote address
    -x, --through <address>             send through ip or address
    -i, --interface <device>            bind to interface
    -a, --listen-transport <options>    listen transport
    -b, --remote-transport <options>    remote transport

LOG OPTIONS:
        --log-level <level>    override log level
        --log-output <path>    override log output

DNS OPTIONS:
        --dns-mode <mode>            override dns mode
        --dns-min-ttl <second>       override dns min ttl
        --dns-max-ttl <second>       override dns max ttl
        --dns-cache-size <number>    override dns cache size
        --dns-protocol <protocol>    override dns protocol
        --dns-servers <servers>      override dns servers

PROXY OPTIONS:
        --send-proxy                       send proxy protocol header
        --send-proxy-version <version>     send proxy protocol version
        --accept-proxy                     accept proxy protocol header
        --accept-proxy-timeout <second>    accept proxy protocol timeout

TIMEOUT OPTIONS:
        --tcp-timeout <second>    override tcp timeout
        --udp-timeout <second>    override udp timeout

SUBCOMMANDS:
    convert    convert your legacy configuration into an advanced one
```

Start from command line arguments:

```shell
realm -l 0.0.0.0:5000 -r 1.1.1.1:443
```

Start with a config file:

```shell
# use toml
realm -c config.toml

# use json
realm -c config.json
```

Start with environment variables:

```shell
REALM_CONF='{"endpoints":[{"local":"127.0.0.1:5000","remote":"1.1.1.1:443"}]}' realm

# or
export REALM_CONF=`cat config.json | jq -c `
realm
```

Convert a legacy config file:

```shell
realm convert old.json
```

## Configuration

TOML Example

```toml
[log]
level = "warn"
output = "/var/log/realm.log"

[network]
use_udp = true
zero_copy = true

[[endpoints]]
listen = "0.0.0.0:5000"
remote = "1.1.1.1:443"

[[endpoints]]
listen = "0.0.0.0:10000"
remote = "www.google.com:443"

```

<details>
<summary>JSON Example</summary>
<p>

```json
{
  "log": {
    "level": "warn",
    "output": "/var/log/realm.log"
  },
  "network": {
    "use_udp": true,
    "zero_copy": true
  },
  "endpoints": [
    {
      "listen": "0.0.0.0:5000",
      "remote": "1.1.1.1:443"
    },
    {
      "listen": "0.0.0.0:10000",
      "remote": "www.google.com:443"
    }
  ]
}

```

</p>
</details>

[See other examples here](./examples)

## Overview

```shell
├── log
│   ├── level
│   └── output
├── dns
│   ├── mode
│   ├── protocol
│   ├── nameservers
│   ├── min_ttl
│   ├── max_ttl
│   └── cache_size
├── network
│   ├── use_udp
│   ├── zero_copy
│   ├── fast_open
│   ├── tcp_timeout
│   ├── udp_timeout
│   ├── send_proxy
│   ├── send_proxy_version
│   ├── accept_proxy
│   └── accept_proxy_timeout
└── endpoints
    ├── listen
    ├── remote
    ├── through
    ├── interface
    ├── listen_transport
    ├── remote_transport
    └── network
```

You should provide at least [endpoint.listen](#endpointlisten-string) and [endpoint.remote](#endpointremote-string), the left fields will take their default values.

Option priority: cmd override > endpoint config > global config

### log

#### log.level: string

values:

- off
- error
- warn
- info
- debug
- trace

default: off

#### log.output: string

values:

- stdout
- stderr
- path (e.g. `/var/log/realm.log`)

default: stdout

### dns

Require the `trust-dns` feature

#### dns.mode: string

Dns resolve strategy.

values:

- ipv4_only
- ipv6_only
- ipv4_then_ipv6
- ipv6_then_ipv4
- ipv4_and_ipv6

default: ipv4_and_ipv6

#### dns.protocol: string

Dns transport protocol.

values:

- tcp
- udp
- tcp_and_udp

default: tcp_and_udp

#### dns.nameservers: string array

Custom upstream servers.

format: ["server1", "server2" ...]

default:

If on **unix/windows**, read from the default location.(e.g. `/etc/resolv.conf`).

Otherwise, use google's public dns(`8.8.8.8:53`, `8.8.4.4:53` and `2001:4860:4860::8888:53`, `2001:4860:4860::8844:53`).

#### dns.min_ttl: unsigned int

The minimum lifetime of a positive dns cache

default: 0

#### dns.max_ttl: unsigned int

The maximum lifetime of a positive dns cache

default: 86400 (1 day)

#### dns.cache_size: unsigned int

The maximum count of dns cache

default: 32

### network

#### network.use_udp: bool

Require the `udp` feature

Start listening on a udp endpoint and forward packets to the remote peer.

It will dynamically allocate local endpoints and establish udp associations. Once timeout, the endpoints will be deallocated and the association will be terminated. See also: [network.udp_timeout](#networkudp_timeout-unsigned-int)

Due to the receiver side not limiting access to the association, the relay works like a full-cone NAT.

default: false

#### network.zero_copy: bool

Require the `zero-copy` feature

Use `splice` instead of `send/recv` while handing tcp connection. This will save a lot of memory copies and context switches.

default: false

#### network.fast_open: bool

Require the `fast-open` feature

It is not recommended to enable this option, see [The Sad Story of TCP Fast Open](https://squeeze.isobar.com/2019/04/11/the-sad-story-of-tcp-fast-open/).

default: false

#### network.tcp_tomeout: unsigned int

Close the connection if the peer does not send any data during `timeout`.

***This option remains unimplemented! (since ce5213)***

To disable timeout, you need to explicitly set timeout value to 0.

default: 300

#### network.udp_timeout: unsigned int

Terminate udp association after `timeout`.

The timeout value must be properly configured in case of memory leak. Do not use a large `timeout`!

default: 30

#### network.send_proxy: bool

Requires the `proxy-protocol` feature

Send haproxy PROXY header once the connection established. Both `v1` and `v2` are supported, see [send_proxy_version](#networksend_proxy_version-unsigned-int).

You should make sure the remote peer also speaks proxy-protocol.

default: false

#### network.send_proxy_version: unsigned int

Requires the `proxy-protocol` feature

This option has no effect unless [send_proxy](#networksend_proxy-bool) is enabled.

value:

- 1
- 2

default: 2

#### network.accept_proxy: bool

Requires the `proxy-protocol` feature

Wait for PROXY header once the connection established.

If the remote sender does not send a `v1` or `v2` header before other contents, the connection will be closed.

default: false

### endpoint

#### endpoint.listen: string

Local address, supported formats:

- ipv4:port
- ipv6:port

#### endpoint.remote: string

Remote address, supported formats:

- ipv4:port
- ipv6:port
- example.com:port

#### endpoint.through: string

TCP: Bind a specific `ip` before opening a connection

UDP: Bind a specific `ip` or `address` before sending packet

Supported formats:

- ipv4/ipv6 (tcp/udp)
- ipv4/ipv6:port (udp)

#### endpoint.interface: string

Bind to a specific interface

#### endpoint.listen_transport: string

Require the `transport` feature

See [Kaminari Options](https://github.com/zephyrchien/kaminari#options)

#### endpoint.remote_transport: string

Require the `transport` feature

See [Kaminari Options](https://github.com/zephyrchien/kaminari#options)

#### endpoint.network

The same as [network](#network), override global options.
