![realm](https://github.com/zhboner/realm/workflows/realm/badge.svg)

[中文说明](https://zhb.me/realm)

<p align="center"><img src="https://raw.githubusercontent.com/zhboner/realm/master/realm.png"/></p>

## Introduction

realm is a simple, high performance relay server written in rust.

## Features
- Zero configuration. Setup and run in one command.
- Concurrency. Bidirectional concurrent traffic leads to high performance.
- Low resources cost.

## Usage
```shell
Realm 1.x
A high efficiency proxy tool

USAGE:
    realm [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -u, --udp        enable udp
    -V, --version    Prints version information

OPTIONS:
    -c, --config <path>    use config file
    -l, --listen <addr>    listen address
    -r, --remote <addr>    remote address
```

Start from command line arguments:
```shell
realm -l 127.0.0.1:5000 -r 1.1.1.1:443 --udp
```

Use a config file:
```shell
realm -c config.json
```
```json
{
	"dns_mode": "ipv4_only",
	"endpoints": [
		{
			"local": "0.0.0.0:5000",
			"remote": "1.1.1.1:443",
			"udp": false
		}
        	{
			"local": "0.0.0.0:10000",
			"remote": "www.google.com:443",
			"udp": true
		}
	]
}
```
dns_mode:
```
ipv4_only|ipv6_only|ipv4_and_ipv6|ipv4_then_ipv6|ipv6_then_ipv4
```

## Custom Build
Available Options:
- udp
- tfo
- zero-copy

```shell
# simple tcp
cargo build --release --no-default-features

# enable other options
cargo build --release --no-default-features --features udp, tfo, zero-copy
```
