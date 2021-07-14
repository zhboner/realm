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
```bash
realm -c config.json
```
>example.json
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
>dns_mode
```
ipv4_only|ipv6_only|ipv4_and_ipv6|ipv4_then_ipv6|ipv6_then_ipv4
```
