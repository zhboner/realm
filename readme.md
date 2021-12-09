![realm](https://github.com/zephyrchien/realm/workflows/ci/badge.svg)
![realm](https://github.com/zephyrchien/realm/workflows/release/badge.svg)

[中文说明](https://zhb.me/realm)

<p align="center"><img src="https://raw.githubusercontent.com/zhboner/realm/master/realm.png"/></p>

## Introduction

realm is a simple, high performance relay server written in rust.

## Features
- ~~Zero configuration.~~ Setup and run in one command.
- Concurrency. Bidirectional concurrent traffic leads to high performance.
- Low resources cost.

## Custom Build
available Options:
- udp *(enabled by default)*
- trust-dns *(enabled by default)*
- zero-copy *(enabled on linux)*
- multi-thread *(enabled by default)*
- tfo
- mi-malloc
- jemalloc

see also: `Cargo.toml`

```shell
# simple tcp
cargo build --release --no-default-features

# enable other options
cargo build --release --no-default-features --features udp, tfo, zero-copy, trust-dns
```

## Usage
```
Realm 1.5.x [udp][zero-copy][trust-dns][multi-thread]

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
    -c, --config <path>     use config file
    -l, --listen <addr>     listen address
    -r, --remote <addr>     remote address
    -x, --through <addr>    send through ip or address

GLOBAL OPTIONS:
        --log-level <level>          override log level
        --log-output <path>          override log output
        --dns-mode <mode>            override dns mode
        --dns-protocol <protocol>    override dns protocol
        --dns-servers <servers>      override dns servers
        --tcp-timeout <second>       override tcp timeout
        --udp-timeout <second>       override udp timeout
```

start from command line arguments:
```shell
realm -l 0.0.0.0:5000 -r 1.1.1.1:443
```

start from config file:
```shell
# use toml
realm -c config.toml

# use json
realm -c config.json
```

start from environment variable:
```shell
CONFIG='{"endpoints":[{"local":"127.0.0.1:5000","remote":"1.1.1.1:443"}]}' realm

export CONFIG=`cat config.json | jq -c `
realm
```

## Configuration
TOML Example
```toml
[log]
level = "warn"
output = "/var/log/realm.log"

[dns]
mode = "ipv4_only"
protocol = "tcp_and_udp"
nameservers = ["8.8.8.8:53", "8.8.4.4:53"]

[network]
use_udp = true
zero_copy = true
fast_open = true
tcp_timeout = 300
udp_timeout = 30

[[endpoints]]
listen = "0.0.0.0:5000"
remote = "1.1.1.1:443"

[[endpoints]]
listen = "0.0.0.0:10000"
remote = "www.google.com:443"
through = "0.0.0.0"
```

<details>
<summary>JSON Example</summary>
<pre>
<code>{
	"log": {
		"level": "warn",
		"output": "/var/log/realm.log"
	},
	"dns": {
		"mode": "ipv4_only",
		"protocol": "tcp_and_udp",
		"nameservers": ["8.8.8.8:53", "8.8.4.4:53"]
	},
	"network": {
		"use_udp": true,
		"fast_open": true,
		"zero_copy": true,
		"tcp_timeout": 300,
		"udp_timeout": 30,
	},
	"endpoints": [
		{
			"listen": "0.0.0.0:5000",
			"remote": "1.1.1.1:443"
		},
		{
			"listen": "0.0.0.0:10000",
			"remote": "www.google.com:443",
			"through": "0.0.0.0"
		}
	]
}</code>
</pre>
</details>

## global
```shell
├── log
│   ├── level
│   └── output
├── dns
│   ├── mode
│   ├── protocol
│   └── nameservers
├── network
│   ├── use_udp
│   ├── fast_open
│   ├── zero_copy
│   ├── tcp_timeout
│   └── udp_timeout
└── endpoints
    ├── listen
    ├── remote
    ├── through
    └── network
        ├── use_udp
        ├── fast_open
        ├── zero_copy
        ├── tcp_timeout
        └── udp_timeout
```

You should provide at least `endpoint.listen` and `endpoint.remote`, other fields will take their default values if not provided.

Priority: cmd override > endpoint config > global config

---
### log

#### log.level
- off *(default)*
- error
- info
- debug
- trace

#### log.output
- stdout *(default)*
- stderr
- path, e.g. (`/var/log/realm.log`)

---
### dns
~~this is compatibe with old versions(before `v1.5.0-rc3`), you could still set lookup strategy with `"dns": "ipv4_only"`, which is equal to `"dns": {"mode": "ipv4_only"}`~~ You must use `dns.mode` instead of `dns_mode`

#### dns.mode
- ipv4_only
- ipv6_only
- ipv4_then_ipv6
- ipv6_then_ipv4
- ipv4_and_ipv6 *(default)*

#### dns.protocol
- tcp
- udp
- tcp_and_udp *(default)*

#### dns.nameservers
format: ["server1", "server2" ...]

default:
On **unix/windows**, it will read from the default location.(e.g. `/etc/resolv.conf`). Otherwise use google's public dns as default upstream resolver(`8.8.8.8:53`, `8.8.4.4:53` and `2001:4860:4860::8888:53`, `2001:4860:4860::8844:53`).

---
### network
- use_udp (default: false)
- zero_copy (default: false)
- fast_open (default: false)
- tcp_timeout (default: 300)
- udp_timeout (default: 30)

To disable timeout, you need to explicitly set timeout value as 0

---
### endpoint objects
- local: listen address
- remote: remote address
- through: send through specified ip or address
- network: override global network config
