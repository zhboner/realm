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
```shell
Realm 1.5.0-rc6 [udp][zero-copy][trust-dns]

A high efficiency relay tool

USAGE:
    realm [FLAGS] [OPTIONS]

FLAGS:
    -u, --udp       enable udp forward
    -f, --tfo       enable tcp fast open
    -z, --splice    enable tcp zero copy
    -d, --daemon    run as a unix daemon

OPTIONS:
    -h, --help                    show help
    -v, --version                 show version
    -c, --config <path>           use config file
    -l, --listen <addr>           listen address
    -r, --remote <addr>           remote address
    -x, --through <addr>          send through ip or address
        --tcp-timeout <second>    set timeout value for tcp
        --udp-timeout <second>    set timeout value for udp

GLOBAL OPTIONS:
        --log-level <level>          override log level
        --log-output <path>          override log output
        --dns-mode <mode>            override dns mode
        --dns-protocol <protocol>    override dns protocol
        --dns-servers <servers>     override dns servers
```

start from command line arguments:
```shell
# enable udp
realm -l 127.0.0.1:5000 -r 1.1.1.1:443 --udp

# specify outbound ip
realm -l 127.0.0.1:5000 -r 1.1.1.1:443 --through 127.0.0.1
```

or use a config file:
```shell
realm -c config.json
```

## Configuration
TOML Example
```toml
[log]
level = "warn"
output = "/var/log/realm.log"

[dns_mode]
mode = "ipv4_only"
protocol = "tcp_and_udp"
nameservers = ["8.8.8.8:53", "8.8.4.4:53"]

[[endpoints]]
local = "0.0.0.0:5000"
remote = "1.1.1.1:443"

[[endpoints]]
udp = true
fast_open = true
zero_copy = false
tcp_timeout = 300
udp_timeout = 30
local = "0.0.0.0:10000"
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
	"dns_mode": {
		"mode": "ipv4_only",
		"protocol": "tcp_and_udp",
		"nameservers": ["8.8.8.8:53", "8.8.4.4:53"]
	},
	"endpoints": [
		{
			"local": "0.0.0.0:5000",
			"remote": "1.1.1.1:443"
		},
		{
			"udp": true,
			"fast_open": true,
			"zero_copy": true,
			"tcp_timeout": 300,
			"udp_timeout": 30,
			"local": "0.0.0.0:10000",
			"remote": "www.google.com:443",
			"through": "0.0.0.0"
		}
	]
}</code>
</pre>
</details>

## global: [log, dns, endpoints]
Note: must provide `endpoint.local` and `endpoint.remote`

---
### log: [level, output]

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
### dns: [mode, protocol, nameservers]
this is compatibe with old versions(before `v1.5.0-rc3`), you could still set lookup priority with `"dns_mode": "ipv4_only"`, which is equal to `"dns_mode": {"mode": "ipv4_only"}`

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
On **unix/windows**, it will read from the default location.(e.g. `/etc/resolv.conf`). Otherwise use google's public dns as default upstream resolver(`8.8.8.8`, `8.8.4.4` and `2001:4860:4860::8888`, `2001:4860:4860::8844`).

---
### endpoint objects
- local:       listen address
- remote:      remote address
- through:     send through specified ip or address, this is optional
- udp:         true|false, default=false
- zero_copy:   true|false, default=false
- fast_open:   true|false, default=false
- tcp_timeout: tcp timeout(sec), default=300
- udp_timeout: udp timeout(sec), default=30
