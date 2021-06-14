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

This executable takes 1 arguments:

- -L [--listen] listen config, can be configured multi times. [bind_address]:port/[host]:hostport

An example to listen on port 30000 and forwarding traffic to example.com:12345 is as follows.

```bash
./realm -L=127.0.0.1:30000/example.com:12345
```

An example to listen on port 30000 and forwarding traffic to example.com:12345 is as follows, to listen on port 40000 and forwarding traffic to example.com:23456 is as follows at the same time.

```bash
./realm -L=127.0.0.1:30000/example.com:12345 -L=127.0.0.1:40000/example.com:23456
```
