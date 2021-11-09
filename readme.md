![realm](https://github.com/zephyrchien/realm/workflows/ci/badge.svg)
![realm](https://github.com/zephyrchien/realm/workflows/release/badge.svg)

[中文说明](https://zhb.me/realm)

<p align="center"><img src="https://raw.githubusercontent.com/zhboner/realm/master/realm.png"/></p>

## Introduction

realm is a simple, high performance relay server written in rust.

## Features
- Zero configuration. Setup and run in one command.
- Concurrency. Bidirectional concurrent traffic leads to high performance.
- Low resources cost.

## Custom Build
available Options:
- udp *(enabled)*
- trust-dns *(enabled)*
- zero-copy *(enabled on linux)*
- tfo *(disabled)*

```shell
# simple tcp
cargo build --release --no-default-features

# enable other options
cargo build --release --no-default-features --features udp, tfo, zero-copy, trust-dns
```

## Usage
```shell
Realm 1.x
A high efficiency proxy tool

USAGE:
    realm [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
    -d, --daemon       daemonize
    -f, --tfo          enable tfo
    -u, --udp          enable udp
    -z, --zero-copy    enable tcp zero-copy
    -h, --help         Prints help information
    -V, --version      Prints version information

OPTIONS:
    -c, --config  <path>    use config file
    -l, --listen  <addr>    listen address
    -r, --remote  <addr>    remote address
    -x, --through <addr>    send through specific ip or address
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

<details>
<summary>Example</summary>
<pre>
<code>{
	"dns_mode": {
		"mode": "ipv4_only",
		"protocol": "tcp+udp",
		"nameservers": ["8.8.8.8", "8.8.4.4"]
	},
	"endpoints": [
		{
			"local": "0.0.0.0:5000",
			"remote": "1.1.1.1:443"
		},
		{
			"local": "0.0.0.0:10000",
			"remote": "www.google.com:443",
			"udp": true,
			"fast_open": true,
			"zero_copy": true
		},
		{
			"local": "0.0.0.0:15000",
			"remote": "www.microsoft.com:443",
			"through": "127.0.0.1"
		}
	]
}</code>
</pre>
</details>

### dns
this is compatibe with old versions(before `v1.5.0-rc3`), you could still set lookup priority with `"dns_mode": "ipv4_only"`, which is equal to `"dns_mode": {"mode": "ipv4_only"}`

#### mode
- ipv4_only
- ipv6_only
- ipv4_then_ipv6 *(default)*
- ipv6_then_ipv4
- ipv4_and_ipv6

#### protocol
- tcp
- udp
- tcp+udp *(default)*

#### nameservers
format: ["server1", "server2" ...]

default:
On **unix/windows**, it will read from the default location.(e.g. `/etc/resolv.conf`). Otherwise use google's public dns as default upstream resolver(`8.8.8.8`, `8.8.4.4` and `2001:4860:4860::8888`, `2001:4860:4860::8844`).

### endpoint objects
- local *(listen address)*
- remote *(remote address)*
- through *(send through specified ip or address, this is optional)*
- udp *(true|false, default=false)*
- zero_copy *(true|false, default=false)*
- fast_open *(true|false, default=false)*
