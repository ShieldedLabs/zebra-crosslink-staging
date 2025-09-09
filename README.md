# zebra-crosslink

<!-- Mark to add banner and badges -->

## Table of Contents

- [Background](#background)
- [Install](#install)
- [Usage](#usage)
- [Maintainers](#maintainers)
- [Contributing](#contributing)
- [License](#license)

## Background

`zebra-crosslink` is Shielded Labs's implementation of *Crosslink*, a hybrid PoW/PoS consensus
protocol for [Zcash](https://z.cash/).

**Status:** This codebase is an early prototype, and suitable only for contributors who want to get
involved.

See `./book/src/crosslink.md` for in-progress documentation on this implementation.

See `./SCOPING.md` for major goals and non-goals for this implementation.

## Install

Building zebra-crosslink requires [Rust](https://www.rust-lang.org/tools/install),
[libclang](https://clang.llvm.org/doxygen/group__CINDEX.html), and a C++ compiler.

### Dependencies

1. Install [`cargo` and `rustc`](https://www.rust-lang.org/tools/install).

2. Install zebra-crosslink's build dependencies:

   - **libclang** is a library that might have different names depending on your
     package manager. Typical names are `libclang`, `libclang-dev`, `llvm`, or
     `llvm-dev`.
   - **clang** or another C++ compiler: `g++` (all platforms) or `Xcode` (macOS).
   - **[`protoc`](https://grpc.io/docs/protoc-installation/)**

```sh
cargo build --locked zebrad
```

## Usage

```sh
cargo run
```

To connect to the current prototype testnet, see [crosslink-testnet.md](crosslink-testnet.md).

## Maintainers

zebra-crosslink is maintained by Shielded Labs, makers of fine Zcash software.

## Contributing

Our github issues are open for feedback. We will accept pull requests after the [prototyping phase
is done](https://ShieldedLabs.net/roadmap).

## License

Zebra is distributed under the terms of both the MIT license and the Apache
License (Version 2.0). Some Zebra crates are distributed under the [MIT license
only](LICENSE-MIT), because some of their code was originally from MIT-licensed
projects. See each crate's directory for details.

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT).
