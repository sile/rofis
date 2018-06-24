rofis
=====

[![rofis](http://meritbadge.herokuapp.com/rofis)](https://crates.io/crates/rofis)
[![Documentation](https://docs.rs/rofis/badge.svg)](https://docs.rs/rofis)
[![Build Status](https://travis-ci.org/sile/rofis.svg?branch=master)](https://travis-ci.org/sile/rofis)
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
