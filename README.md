# libtor

`libtor` is a Rust crate for bundling inside your project a fully-running Tor daemon.
It exposes a nicer interface to operate it compared to the bare-bones version, [libtor-sys](https://crates.io/crates/libtor-sys).
If you need further instructions on how to build or cross-compile the project, you should refer to the [`libtor-sys` README.md](https://github.com/MagicalBitcoin/libtor-sys).

`libtor` makes it easier for projects on multiple platforms to use Tor without depending on the user to configure complex proxy settings from other external software.

## Example

```
use libtor::{Tor, TorFlag, TorAddress, HiddenServiceVersion};

Tor::new()
    .flag(TorFlag::DataDirectory("/tmp/tor-rust".into()))
    .flag(TorFlag::SocksPort(19050))
    .flag(TorFlag::HiddenServiceDir("/tmp/tor-rust/hs-dir".into()))
    .flag(TorFlag::HiddenServiceVersion(HiddenServiceVersion::V3))
    .flag(TorFlag::HiddenServicePort(TorAddress::Port(8000), None.into()))
    .start()?;
```

Since Tor uses internally some static variables to keep its state, keep in mind that **you can't start more than one Tor instance per process**.

## Supported platforms

The currently supported platforms are:

* Linux (tested on Fedora 30 and Ubuntu Xenial)
* Android through the NDK
* MacOS
* iOS
* Windows cross-compiled from Linux with `mingw`

Coming Soon :tm::

* Windows (natively built)

## Dependencies

The following dependencies are needed:
- `openssl`
- `pkg-config`
- `file`
- the "usual" C build tools: a compiler, `automake`, `autoconf`
