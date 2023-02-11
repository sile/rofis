rofis
=====

[![rofis](https://img.shields.io/crates/v/rofis.svg)](https://crates.io/crates/rofis)
[![Actions Status](https://github.com/sile/rofis/workflows/CI/badge.svg)](https://github.com/sile/rofis/actions)
![License](https://img.shields.io/crates/l/rofis)

A read-only, puny HTTP file server.

Install
-------

```console
$ cargo install rofis
```

Example
--------

```console
$ rofis --daemonize
$ curl http://localhost:8080/README.md
```
