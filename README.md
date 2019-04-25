# Netlink-Rust

Netlink library for rust. Work in progress.

## Background

This was made to explore Rust while also exploring how nl80211 works. Thus a
netlink subsystem was needed.

As most API's in the Linux kernel realm, this is a bit hairy and incoherent.
Most of the energy has been put into the generic Netlink area.

## Examples

There are some examples in the examples directory. Run them using cargo.

```
cargo run --example example
cargo run --example uevent_example
```

## Compatability

Rust 1.30.0 or later is needed.

Tested on following platforms,
 - Linux 4.18 x86_64, Fedora 28
 - Linux 4.1 ARMv7
 - Linux 4.9 AArch64

## License

Licensed under the MIT license.

[![Build Status](https://travis-ci.org/blueluna/netlink-rs.svg?branch=master)](https://travis-ci.org/blueluna/netlink-rs) [![Crates.io](https://img.shields.io/crates/v/netlink-rust.svg)](https://crates.io/crates/netlink-rust)
