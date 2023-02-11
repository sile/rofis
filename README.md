rofis
=====

[![rofis](https://img.shields.io/crates/v/rofis.svg)](https://crates.io/crates/rofis)
[![Documentation](https://docs.rs/rofis/badge.svg)](https://docs.rs/rofis)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

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
