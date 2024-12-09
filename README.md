# `tosca-app`

[![LICENSE][license badge]][license]

**tosca-app** is a web app for managing
[tosca](https://github.com/ToscaLabs/tosca/) devices.

## Building

To build this crate with a `debug` profile run:

```console
cargo build
```

To build this crate with a `release` profile which enables all time and
memory optimizations run:

```console
cargo build --release
```

To build without `logging` feature

```console
cargo build --no-default-features
```

To build with errors and messages in `Italian` language

```console
cargo build --features italian
```

<!-- Links -->
[license]: https://github.com/ToscaLabs/tosca-app/blob/master/LICENSE

<!-- Badges -->
[license badge]: https://img.shields.io/badge/license-MIT-blue.svg
