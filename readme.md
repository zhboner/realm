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
This exacutable takes 2 arguments:
- -l [--local] local socket address. Default address 127.0.0.1 is used when the address is omitted.
- -r [--remote] remote socker address. Both domain and ip address are accepted. If a domain is passed, the resolver will try to resolve and update the ip address regularly, ipv4 is preferred.

An example to listen on port 30000 and forwarding traffic to example.com:12345 is as follows.
```
./realm -l 127.0.0.1:30000 -r example.com:12345
```
